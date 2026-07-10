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