//! Extraccion de librerias nativas hacia la carpeta natives de la instancia.
//!
//! Usa los jars de natives que el instalador ya descargo dentro del propio
//! directorio natives. Recorre las librerias de la version, detecta cuales
//! tienen native para el SO actual, ubica su jar y extrae los binarios
//! (.dll en Windows; .so/.dylib preparados para Linux/Mac).

use crate::error::{AetherError, Result};
use crate::minecraft::version::VersionDetail;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Extension del binario nativo segun el SO actual.
fn native_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        ".dll"
    } else if cfg!(target_os = "macos") {
        ".dylib"
    } else {
        ".so"
    }
}

/// Reconstruye el nombre del jar de natives tal como lo guardo el instalador:
/// nombre hoja del `artifact.path`, o "<classifier>.jar" como fallback.
fn native_jar_filename(classifier: &str, art_path: &Option<String>) -> String {
    let fname = art_path
        .clone()
        .unwrap_or_else(|| format!("{classifier}.jar"));
    Path::new(&fname)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{classifier}.jar"))
}

/// Extrae un jar de natives dentro de `dest_dir` (plano). Solo escribe los
/// binarios nativos del SO actual; ignora directorios y META-INF.
fn extract_jar(jar_path: &Path, dest_dir: &Path) -> Result<u32> {
    let file = fs::File::open(jar_path)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| {
        AetherError::InvalidState(format!("zip invalido {}: {e}", jar_path.display()))
    })?;
    let ext = native_extension();
    let mut count = 0u32;

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| AetherError::InvalidState(format!("entrada zip {i}: {e}")))?;

        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if name.starts_with("META-INF/") {
            continue;
        }
        if !name.to_ascii_lowercase().ends_with(ext) {
            continue;
        }
        // Aplanar: descartar cualquier ruta interna y quedarse con el archivo.
        let leaf = match Path::new(&name).file_name() {
            Some(n) => n.to_owned(),
            None => continue,
        };

        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        fs::write(dest_dir.join(leaf), &buf)?;
        count += 1;
    }
    Ok(count)
}

/// Extrae todas las natives permitidas de una version al directorio natives
/// de la instancia. Idempotente: sobreescribe los binarios existentes.
/// Devuelve la cantidad de binarios extraidos.
pub fn extract_natives(detail: &VersionDetail, natives_dir: &Path) -> Result<u32> {
    fs::create_dir_all(natives_dir)?;
    let mut total = 0u32;

    for lib in &detail.libraries {
        // Respetar las reglas allow/disallow por SO.
        if !lib.allowed() {
            continue;
        }
        // Solo librerias con native para el SO actual (p.ej. "natives-windows").
        let classifier = match lib.native_classifier() {
            Some(c) => c,
            None => continue,
        };

        // Ruta del jar de natives que dejo el instalador dentro de natives_dir.
        let art_path = lib
            .downloads
            .as_ref()
            .and_then(|dl| dl.classifiers.as_ref())
            .and_then(|c| c.get(&classifier))
            .and_then(|art| art.path.clone());

        let jar_path = natives_dir.join(native_jar_filename(&classifier, &art_path));
        if !jar_path.exists() {
            // No lo bajo el instalador o no aplica: se ignora en silencio.
            continue;
        }

        total += extract_jar(&jar_path, natives_dir)?;
    }
    Ok(total)
}
