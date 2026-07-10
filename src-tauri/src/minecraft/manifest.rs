//! Version manifest oficial de Mojang: indice global de versiones y
//! resolucion de la URL del JSON detallado de cada version.

use crate::error::{AetherError, Result};
use serde::{Deserialize, Serialize};

const MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
}

impl VersionManifest {
    pub async fn fetch(client: &reqwest::Client) -> Result<Self> {
        Ok(client
            .get(MANIFEST_URL)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub fn find(&self, id: &str) -> Result<&VersionEntry> {
        self.versions
            .iter()
            .find(|v| v.id == id)
            .ok_or_else(|| AetherError::NotFound(format!("version {id}")))
    }
}
