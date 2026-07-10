//! JSON detallado de una version: librerias (con reglas por SO), assets,
//! client jar, natives y hint de Java recomendado.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDetail {
    pub id: String,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexRef,
    pub assets: String,
    pub downloads: ClientDownloads,
    pub libraries: Vec<Library>,
    #[serde(rename = "javaVersion", default)]
    pub java_version: Option<JavaVersion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JavaVersion {
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub url: String,
    pub sha1: String,
    #[serde(default)]
    pub total_size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientDownloads {
    pub client: Artifact,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    pub url: String,
    pub sha1: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub natives: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<std::collections::HashMap<String, Artifact>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<OsRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
}

/// Nombre del SO segun la convencion de Mojang.
pub fn current_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

impl Library {
    /// Evalua las reglas allow/disallow para el SO actual.
    pub fn allowed(&self) -> bool {
        if self.rules.is_empty() {
            return true;
        }
        let mut allow = false;
        for rule in &self.rules {
            let matches = match &rule.os {
                Some(os) => os.name.as_deref().map_or(true, |n| n == current_os()),
                None => true,
            };
            if matches {
                allow = rule.action == "allow";
            }
        }
        allow
    }

    /// Clave de native para el SO actual (p.ej. "natives-windows"), si aplica.
    pub fn native_classifier(&self) -> Option<String> {
        self.natives.as_ref().and_then(|m| m.get(current_os()).cloned())
    }
}

/// Objeto individual del asset index.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub objects: std::collections::HashMap<String, AssetObject>,
}

/// Resumen serializable de una version para la UI (selector).
#[derive(Debug, Clone, Serialize)]
pub struct VersionOption {
    pub id: String,
    pub kind: String,
    pub release_time: String,
}
