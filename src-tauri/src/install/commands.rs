//! Comandos IPC de instalacion. Gestiona el ciclo de estados y la
//! cancelacion mediante CancellationToken por instancia.

use crate::error::{AetherError, Result};
use crate::install::{install_for, InstallEvent};
use crate::instance::{store, InstallStatus};
use crate::AppState;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

/// Registro global de instalaciones en curso (id -> token de cancelacion).
#[derive(Default)]
pub struct InstallRegistry {
    pub active: Mutex<HashMap<String, CancellationToken>>,
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn set_status(state: &AppState, id: &str, status: InstallStatus) {
    let mut index = state.index.lock().unwrap();
    if let Some(s) = index.iter_mut().find(|s| s.id == id) {
        s.status = status;
    }
    let _ = store::save_index(&index);
}

#[tauri::command]
pub async fn install_instance(
    state: State<'_, AppState>,
    registry: State<'_, InstallRegistry>,
    id: String,
    on_event: Channel<InstallEvent>,
) -> Result<()> {
    let mut inst = store::load_instance(&id)?;

    let cancel = CancellationToken::new();
    registry.active.lock().unwrap().insert(id.clone(), cancel.clone());

    inst.status = InstallStatus::Installing;
    inst.installation.current_phase = "starting".into();
    inst.installation.progress = 0.0;
    inst.last_error = None;
    let _ = store::save_instance(&inst);
    set_status(&state, &id, InstallStatus::Installing);

    let result = install_for(&state.http, &inst, &on_event, cancel.clone()).await;
    registry.active.lock().unwrap().remove(&id);

    match result {
        Ok(bytes) => {
            inst.status = InstallStatus::Ready;
            inst.installed_at = Some(now());
            inst.total_size_bytes = bytes;
            inst.installation.current_phase = "done".into();
            inst.installation.progress = 1.0;
            let _ = store::save_instance(&inst);
            set_status(&state, &id, InstallStatus::Ready);
            let _ = on_event.send(InstallEvent::Done);
            Ok(())
        }
        Err(_) if cancel.is_cancelled() => {
            inst.status = InstallStatus::Cancelled;
            inst.installation.current_phase = "cancelled".into();
            let _ = store::save_instance(&inst);
            set_status(&state, &id, InstallStatus::Cancelled);
            let _ = on_event.send(InstallEvent::Cancelled);
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            inst.status = InstallStatus::Error;
            inst.last_error = Some(msg.clone());
            inst.installation.current_phase = "error".into();
            let _ = store::save_instance(&inst);
            set_status(&state, &id, InstallStatus::Error);
            let _ = on_event.send(InstallEvent::Failed { message: msg.clone() });
            Err(AetherError::InvalidState(msg))
        }
    }
}

#[tauri::command]
pub async fn cancel_install(registry: State<'_, InstallRegistry>, id: String) -> Result<()> {
    if let Some(token) = registry.active.lock().unwrap().get(&id) {
        token.cancel();
        Ok(())
    } else {
        Err(AetherError::NotFound(format!("no hay instalacion activa para {id}")))
    }
}

/// Lista de versiones de Minecraft para el selector de la UI.
#[tauri::command]
pub async fn list_mc_versions(
    state: State<'_, AppState>,
) -> Result<Vec<crate::minecraft::version::VersionOption>> {
    use crate::minecraft::manifest::VersionManifest;
    let manifest = VersionManifest::fetch(&state.http).await?;
    Ok(manifest
        .versions
        .into_iter()
        .map(|v| crate::minecraft::version::VersionOption {
            id: v.id,
            kind: v.kind,
            release_time: v.release_time,
        })
        .collect())
}
