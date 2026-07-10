//! Punto de entrada único para el resto del launcher: "dame el Java correcto
//! para este major". Encapsula la política detectar-o-descargar para que
//! `launch` no conozca manifests, descargas ni rutas internas.

use crate::error::{AetherError, Result};
use crate::runtime::{detect, install, ManagedRuntime, RuntimeEvent};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

/// Garantiza un runtime de Java administrado para `major`:
/// 1. Si ya existe en disco, lo devuelve (sin red).
/// 2. Si no, lo descarga, lo verifica y lo deja listo.
///
/// `on_event` es opcional: `Some` emite progreso (misma superficie que el
/// comando `download_runtime`); `None` corre silencioso (útil en el launch).
///
/// Errores claros:
/// - runtime incompatible  -> `Runtime` (plataforma/major sin runtime).
/// - descarga fallida      -> se propaga `Http` / `HashMismatch`.
/// - Java inexistente tras descargar -> `Runtime` (desde install::Fase E).
pub async fn ensure_java(
    client: &reqwest::Client,
    major: u32,
    on_event: Option<&Channel<RuntimeEvent>>,
    cancel: CancellationToken,
) -> Result<ManagedRuntime> {
    // 1. Runtime administrado ya presente: sin red.
    if let Some(existing) = detect::find_managed_runtime(major)? {
        if let Some(ch) = on_event {
            let _ = ch.send(RuntimeEvent::Done {
                major,
                java_path: existing.java_path.clone(),
            });
        }
        return Ok(existing);
    }

    // 2. No existe: descargar. `download_runtime` ya valida cada archivo por
    //    SHA1 y verifica que el binario quede disponible al final (Fase E).
    install::download_runtime(client, major, on_event, cancel)
        .await
        .map_err(|e| match e {
            // Índice de Mojang sin runtime para esta plataforma/major.
            AetherError::NotFound(msg) => AetherError::Runtime(format!(
                "no hay un runtime de Java {major} compatible con esta plataforma: {msg}"
            )),
            other => other,
        })
}