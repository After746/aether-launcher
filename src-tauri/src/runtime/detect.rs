//! Deteccion del Java requerido por una version de Minecraft y descubrimiento
//! de runtimes administrados ya presentes en disco.
//!
//! Fuente de verdad: el campo `javaVersion` del version.json de Mojang. Si no
//! existe (versiones pre-1.17), caemos a Java 8, que es justo lo que necesitan
//! esas versiones (LaunchWrapper castea a URLClassLoader, roto en Java 9+).

use crate::error::Result;
use crate::instance::store;
use crate::minecraft::manifest::VersionManifest;
use crate::minecraft::version::VersionDetail;
use crate::runtime::paths;
use crate::runtime::{ManagedRuntime, RequiredJava, RequiredJavaSource};

/// Major de Java al que caemos cuando el version.json no declara `javaVersion`.
const FALLBACK_JAVA_MAJOR: u32 = 8;

/// Lee el version.json cacheado en disco, si el instalador ya lo guardo.
/// Ruta: `<data_dir>/versions/<id>/<id>.json`.
async fn cached_version_detail(mc_version: &str) -> Result<Option<VersionDetail>> {
    let path = store::data_dir()?
        .join("versions")
        .join(mc_version)
        .join(format!("{mc_version}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let raw = tokio::fs::read_to_string(&path).await?;
    let detail: VersionDetail = serde_json::from_str(&raw)?;
    Ok(Some(detail))
}

/// Descarga (solo en memoria) el version.json desde Mojang cuando no esta
/// cacheado. Reutiliza el manifest y los tipos existentes; NO escribe a disco
/// (eso es responsabilidad del instalador, que no tocamos).
async fn remote_version_detail(
    client: &reqwest::Client,
    mc_version: &str,
) -> Result<VersionDetail> {
    let manifest = VersionManifest::fetch(client).await?;
    let entry = manifest.find(mc_version)?;
    let detail: VersionDetail = client
        .get(&entry.url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(detail)
}

/// Resuelve que major de Java necesita una version. Primero intenta el
/// version.json cacheado; si no esta, lo consulta online.
pub async fn required_java(
    client: &reqwest::Client,
    mc_version: &str,
) -> Result<RequiredJava> {
    let detail = match cached_version_detail(mc_version).await? {
        Some(d) => d,
        None => remote_version_detail(client, mc_version).await?,
    };

    Ok(match detail.java_version {
        Some(jv) => RequiredJava {
            major: jv.major_version,
            source: RequiredJavaSource::VersionJson,
        },
        None => RequiredJava {
            major: FALLBACK_JAVA_MAJOR,
            source: RequiredJavaSource::Fallback,
        },
    })
}

/// Busca un runtime administrado ya instalado para un major concreto.
/// En Fase 1 normalmente devuelve None (aun no descargamos), pero deja el
/// descubrimiento listo para que Fase 3 solo tenga que materializar archivos
/// en la ruta que este modulo ya conoce.
pub fn find_managed_runtime(major: u32) -> Result<Option<ManagedRuntime>> {
    let home = paths::runtime_home_for(major)?;
    if !home.exists() {
        return Ok(None);
    }
    match paths::java_executable_in(&home) {
        Some(java_path) => Ok(Some(ManagedRuntime {
            major,
            java_path: java_path.to_string_lossy().to_string(),
            home: home.to_string_lossy().to_string(),
        })),
        None => Ok(None),
    }
}
