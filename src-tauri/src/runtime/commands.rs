//! Superficie IPC del Runtime Manager (Fase 1).
//!
//! Expone la resolucion "que Java necesita esta instancia y lo tengo ya?".
//! Persiste el major detectado en la instancia (campo java_major, que ya
//! existe en el modelo) reutilizando el store actual, sin tocar su logica.

use crate::error::Result;
use crate::instance::store;
use crate::runtime::detect;
use crate::runtime::{RequiredJava, RuntimeResolution};
use crate::AppState;
use tauri::State;
use crate::runtime::{install, ManagedRuntime, RuntimeEvent};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

/// Descarga (si falta) el runtime de Java administrado para `major` y lo deja
/// almacenado en <data_dir>/runtimes/java-<major>/. Si ya existe, lo devuelve
/// sin volver a descargar. NO integra el runtime con el launch (fase futura).
#[tauri::command]
pub async fn download_runtime(
    state: State<'_, AppState>,
    major: u32,
    on_event: Channel<RuntimeEvent>,
) -> Result<ManagedRuntime> {
    // Idempotente: si ya esta, no rebajamos nada.
    if let Some(existing) = detect::find_managed_runtime(major)? {
        let _ = on_event.send(RuntimeEvent::Done {
            major,
            java_path: existing.java_path.clone(),
        });
        return Ok(existing);
    }

    let cancel = CancellationToken::new();
    match install::download_runtime(&state.http, major, Some(&on_event), cancel).await {
        Ok(managed) => Ok(managed),
        Err(e) => {
            let _ = on_event.send(RuntimeEvent::Failed { message: e.to_string() });
            Err(e)
        }
    }
}

/// Resuelve el Java requerido por una instancia (por su id) y comprueba si ya
/// hay un runtime administrado disponible. Persiste el major detectado.
#[tauri::command]
pub async fn resolve_runtime(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<RuntimeResolution> {
    let mut inst = store::load_instance(&instance_id)?;

    let required = detect::required_java(&state.http, &inst.mc_version).await?;
    let managed = detect::find_managed_runtime(required.major)?;
    let ready = managed.is_some();

    // Guardamos el major detectado para reutilizarlo (el campo ya existia
    // en el modelo, reservado justamente para esto).
    if inst.java_major != Some(required.major) {
        inst.java_major = Some(required.major);
        let _ = store::save_instance(&inst);
    }

    Ok(RuntimeResolution { required, managed, ready })
}

/// Variante sin instancia: util para el selector de versiones de la UI, que
/// puede mostrar "requiere Java N" antes de crear la instancia.
#[tauri::command]
pub async fn required_java_for_version(
    state: State<'_, AppState>,
    mc_version: String,
) -> Result<RequiredJava> {
    detect::required_java(&state.http, &mc_version).await
}