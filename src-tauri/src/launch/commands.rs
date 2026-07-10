//! Comando IPC de lanzamiento. Puente entre la UI y launch::launch_instance.
//! Devuelve el PID en caso de exito; los errores viajan como AetherError
//! serializable ({ code, message }).

use crate::error::Result;
use crate::instance::store;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Lanza una instancia ya instalada y devuelve el PID del proceso de Java.
///
/// Diagnostico: stdout y stderr se vuelcan a `<instancia>/minecraft/latest.log`
/// (y se replican por eprintln! para verlos en la consola de `tauri dev`).
/// Sin esto, el motivo del crash quedaba invisible.
#[tauri::command]
pub async fn launch_instance(id: String) -> Result<u32> {
    let inst = store::load_instance(&id)?;
    let launched = super::launch_instance(&inst)?;
    let pid = launched.pid;
    let mut child = launched.child;

    // Ruta del log dentro de la carpeta de la instancia (misma convencion
    // que game_dir: <instance.path>/minecraft).
    let log_path = Path::new(&inst.path).join("minecraft").join("latest.log");

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    std::thread::spawn(move || {
        // Abrimos (o creamos/truncamos) el log una sola vez por lanzamiento.
        let mut log = std::fs::File::create(&log_path).ok();

        // stdout en un hilo aparte para no serializar ambos flujos.
        let out_handle = stdout.map(|out| {
            let log_path = log_path.clone();
            std::thread::spawn(move || {
                let mut log = std::fs::OpenOptions::new().append(true).open(&log_path).ok();
                for line in BufReader::new(out).lines().map_while(std::result::Result::ok) {
                    eprintln!("[mc:out] {line}");
                    if let Some(f) = log.as_mut() {
                        let _ = writeln!(f, "{line}");
                    }
                }
            })
        });

        // stderr: la fuente del error real que buscamos.
        if let Some(err) = stderr {
            for line in BufReader::new(err).lines().map_while(std::result::Result::ok) {
                eprintln!("[mc:err] {line}");
                if let Some(f) = log.as_mut() {
                    let _ = writeln!(f, "{line}");
                }
            }
        }

        if let Some(h) = out_handle {
            let _ = h.join();
        }
        // Esperamos al proceso para conocer el codigo de salida y cerrarlo limpio.
        let status = child.wait();
        if let Some(f) = log.as_mut() {
            let _ = writeln!(f, "[aether] proceso finalizado: {status:?}");
        }
        eprintln!("[aether] proceso finalizado: {status:?}");
    });

    Ok(pid)
}