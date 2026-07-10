//! Indice oficial de runtimes de Java de Mojang (el mismo que consume el
//! launcher oficial). Dos niveles:
//!   1. all.json  -> plataforma -> componente -> release (con URL del manifest)
//!   2. manifest del componente -> listado de archivos individuales
//!
//! Cada archivo trae su URL, sha1 y tamano, asi que el sistema de descargas
//! existente (download::run_batch) sirve sin adaptaciones.

use crate::error::{AetherError, Result};
use crate::minecraft::version::current_os;
use crate::runtime::paths::current_arch;
use serde::Deserialize;
use std::collections::HashMap;

/// URL fija y versionada del indice de runtimes (identica a la del launcher
/// oficial de Minecraft).
const JAVA_RUNTIME_ALL_URL: &str =
    "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// all.json: clave de plataforma -> nombre de componente -> lista de releases.
pub type JavaRuntimeIndex = HashMap<String, HashMap<String, Vec<RuntimeRelease>>>;

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeRelease {
    pub manifest: ManifestRef,
    pub version: RuntimeVersion,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestRef {
    pub sha1: String,
    #[serde(default)]
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeVersion {
    pub name: String,
    #[serde(default)]
    pub released: String,
}

/// Manifest de un componente concreto: cada entrada es un archivo, directorio
/// o symlink relativo a la raiz del runtime.
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeManifest {
    pub files: HashMap<String, FileEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FileEntry {
    File {
        #[serde(default)]
        executable: bool,
        downloads: FileDownloads,
    },
    Directory,
    Link {
        target: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileDownloads {
    /// Version sin comprimir (la usamos: evita depender de un descompresor lzma).
    pub raw: Download,
    #[serde(default)]
    pub lzma: Option<Download>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Download {
    pub sha1: String,
    #[serde(default)]
    pub size: u64,
    pub url: String,
}

/// Clave de plataforma segun la convencion de Mojang (SO + arquitectura).
pub fn mojang_platform_key() -> Result<&'static str> {
    let key = match (current_os(), current_arch()) {
        ("windows", "x64") => "windows-x64",
        ("windows", "arm64") => "windows-arm64",
        ("windows", "x86") => "windows-x86",
        ("linux", "x64") => "linux",
        ("linux", "x86") => "linux-i386",
        ("osx", "x64") => "mac-os",
        ("osx", "arm64") => "mac-os-arm64",
        (os, arch) => {
            return Err(AetherError::NotFound(format!(
                "no hay runtime de Java para {os}/{arch}"
            )))
        }
    };
    Ok(key)
}

/// Componentes candidatos para un major de Java, en orden de preferencia.
fn component_candidates(major: u32) -> &'static [&'static str] {
    match major {
        8 => &["jre-legacy"],
        16 => &["java-runtime-alpha"],
        17 => &["java-runtime-gamma", "java-runtime-beta", "java-runtime-gamma-snapshot"],
        21 => &["java-runtime-delta"],
        _ => &[],
    }
}

/// Descarga y parsea el all.json.
pub async fn fetch_index(client: &reqwest::Client) -> Result<JavaRuntimeIndex> {
    Ok(client
        .get(JAVA_RUNTIME_ALL_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

/// Elige el release adecuado para (plataforma actual, major). Usa la tabla de
/// componentes conocidos y, si el major es desconocido, cae a buscar por el
/// nombre de version (p.ej. "17.0.8" empieza con "17").
pub fn select_release<'a>(
    index: &'a JavaRuntimeIndex,
    platform_key: &str,
    major: u32,
) -> Result<&'a RuntimeRelease> {
    let platform = index.get(platform_key).ok_or_else(|| {
        AetherError::NotFound(format!("plataforma {platform_key} ausente en el indice"))
    })?;

    for name in component_candidates(major) {
        if let Some(releases) = platform.get(*name) {
            if let Some(rel) = releases.first() {
                return Ok(rel);
            }
        }
    }

    // Fallback best-effort para majors no mapeados explicitamente.
    let prefix = major.to_string();
    for releases in platform.values() {
        if let Some(rel) = releases.first() {
            if rel.version.name.starts_with(&prefix) {
                return Ok(rel);
            }
        }
    }

    Err(AetherError::NotFound(format!(
        "no hay runtime de Java {major} para {platform_key}"
    )))
}

/// Descarga y parsea el manifest de un componente.
pub async fn fetch_manifest(
    client: &reqwest::Client,
    url: &str,
) -> Result<RuntimeManifest> {
    Ok(client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}