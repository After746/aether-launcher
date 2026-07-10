//! Modelo de dominio de una instancia de Minecraft.
//! Cada instancia es una entidad autocontenida y persistida como TOML.

pub mod commands;
pub mod store;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
}
impl Default for Loader {
    fn default() -> Self {
        Loader::Vanilla
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStatus {
    Created,
    Installing,
    Cancelled,
    Ready,
    Error,
    Corrupt,
}
impl Default for InstallStatus {
    fn default() -> Self {
        InstallStatus::Created
    }
}

/// Preparado para una fase futura. Aun no se consume funcionalmente.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceSettings {
    #[serde(default)]
    pub resolution_width: Option<u32>,
    #[serde(default)]
    pub resolution_height: Option<u32>,
    #[serde(default)]
    pub fullscreen: bool,
}

/// Preparado para Modrinth / CurseForge. Aun sin implementar.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Modpack {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
}

/// Estado persistido de instalacion, para recuperar si el launcher se cierra.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Installation {
    #[serde(default)]
    pub current_phase: String,
    #[serde(default)]
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    #[serde(default)]
    pub loader: Loader,
    #[serde(default)]
    pub loader_version: Option<String>,
    pub ram_mb: u32,
    pub path: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub java_path: Option<String>,
    /// Major de Java recomendado por la version (lo rellena Fase 3, lo usa Fase 4).
    #[serde(default)]
    pub java_major: Option<u32>,
    #[serde(default)]
    pub favorite: bool,
    pub created_at: i64,
    #[serde(default)]
    pub last_played: Option<i64>,
    #[serde(default)]
    pub playtime_secs: u64,
    #[serde(default)]
    pub mod_count: u32,
    #[serde(default)]
    pub status: InstallStatus,
    #[serde(default)]
    pub installed_at: Option<i64>,
    #[serde(default)]
    pub total_size_bytes: u64,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub settings: InstanceSettings,
    #[serde(default)]
    pub modpack: Modpack,
    #[serde(default)]
    pub installation: Installation,
}

/// Resumen ligero para el indice (instances.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSummary {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub loader: Loader,
    pub ram_mb: u32,
    pub icon: Option<String>,
    pub last_played: Option<i64>,
    pub playtime_secs: u64,
    pub mod_count: u32,
    pub favorite: bool,
    pub status: InstallStatus,
    #[serde(default)]
    pub total_size_bytes: u64,
}

impl From<&Instance> for InstanceSummary {
    fn from(i: &Instance) -> Self {
        Self {
            id: i.id.clone(),
            name: i.name.clone(),
            mc_version: i.mc_version.clone(),
            loader: i.loader,
            ram_mb: i.ram_mb,
            icon: i.icon.clone(),
            last_played: i.last_played,
            playtime_secs: i.playtime_secs,
            mod_count: i.mod_count,
            favorite: i.favorite,
            status: i.status,
            total_size_bytes: i.total_size_bytes,
        }
    }
}
