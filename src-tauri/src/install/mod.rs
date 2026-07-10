//! Trait Installer: contrato comun para todos los loaders. Vanilla se
//! implementa completo; Fabric/Forge/NeoForge quedan preparados.

pub mod commands;
pub mod vanilla;

use crate::error::{AetherError, Result};
use crate::instance::{Instance, Loader};
use serde::Serialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstallEvent {
    Started { total_files: u64, total_bytes: u64 },
    Phase { phase: String },
    Progress {
        files_done: u64,
        total_files: u64,
        bytes_done: u64,
        total_bytes: u64,
        current_file: String,
        speed_bps: u64,
    },
    Done,
    Cancelled,
    Failed { message: String },
}

pub trait Installer {
    async fn install(
        &self,
        client: &reqwest::Client,
        instance: &Instance,
        on_event: &Channel<InstallEvent>,
        cancel: CancellationToken,
    ) -> Result<u64>; // devuelve bytes totales instalados
}

/// Enruta al instalador correcto segun el loader de la instancia.
pub async fn install_for(
    client: &reqwest::Client,
    instance: &Instance,
    on_event: &Channel<InstallEvent>,
    cancel: CancellationToken,
) -> Result<u64> {
    match instance.loader {
        Loader::Vanilla => {
            vanilla::VanillaInstaller
                .install(client, instance, on_event, cancel)
                .await
        }
        Loader::Fabric => Err(AetherError::NotImplemented(
            "el instalador de Fabric llegara en una fase posterior".into(),
        )),
        Loader::Forge => Err(AetherError::NotImplemented(
            "el instalador de Forge llegara en una fase posterior".into(),
        )),
        Loader::NeoForge => Err(AetherError::NotImplemented(
            "el instalador de NeoForge llegara en una fase posterior".into(),
        )),
    }
}
