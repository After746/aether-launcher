//! Comando IPC de lanzamiento. Puente entre la UI y launch::launch_instance.
//! Devuelve el PID en caso de exito; los errores viajan como AetherError
//! serializable ({ code, message }).

use crate::error::Result;
use crate::instance::store;

/// Lanza una instancia ya instalada y devuelve el PID del proceso de Java.
///
/// Aun no gestionamos el proceso (ni logs ni cierre), pero drenamos
/// stdout/stderr en hilos aparte: sin un lector, el buffer del pipe se
/// llenaria (~64 KB) y Minecraft se congelaria al escribir.
#[tauri::command]
pub async fn launch_instance(id: String) -> Result<u32> {
    let inst = store::load_instance(&id)?;
    let launched = super::launch_instance(&inst)?;
    let pid = launched.pid;
    let mut child = launched.child;

    if let Some(mut out) = child.stdout.take() {
        std::thread::spawn(move || {
            let _ = std::io::copy(&mut out, &mut std::io::sink());
        });
    }
    if let Some(mut err) = child.stderr.take() {
        std::thread::spawn(move || {
            let _ = std::io::copy(&mut err, &mut std::io::sink());
        });
    }
    // Mantener vivo el proceso de forma independiente. La gestion real
    // (matar/esperar/estado) llega en una fase posterior.
    std::thread::spawn(move || {
        let _ = child.wait();
    });

    Ok(pid)
}