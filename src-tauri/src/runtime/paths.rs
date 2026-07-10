//! Layout en disco de los runtimes administrados. Todo cuelga del data_dir
//! del launcher para que los runtimes viajen con la instalacion y se
//! reutilicen entre lanzamientos.

use crate::error::Result;
use crate::instance::store;
use std::path::{Path, PathBuf};

/// Raiz donde viven todos los runtimes administrados: `<data_dir>/runtimes/`.
pub fn runtimes_dir() -> Result<PathBuf> {
    let d = store::data_dir()?.join("runtimes");
    std::fs::create_dir_all(&d)?;
    Ok(d)
}

/// Carpeta reservada para un major concreto: `<data_dir>/runtimes/java-<major>/`.
/// Fase 2/3 materializaran el runtime aqui dentro.
pub fn runtime_home_for(major: u32) -> Result<PathBuf> {
    Ok(runtimes_dir()?.join(format!("java-{major}")))
}

/// Nombre del SO segun la convencion de Mojang (reutiliza minecraft::version).
pub fn current_os() -> &'static str {
    crate::minecraft::version::current_os()
}

/// Arquitectura segun la convencion de Mojang (x64, arm64, x86...).
pub fn current_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    }
}

/// Ruta relativa del ejecutable de Java segun el SO (layout de Mojang).
pub fn java_bin_relative() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from("bin").join("java.exe")
    } else if cfg!(target_os = "macos") {
        // Los runtimes de Mojang en macOS usan el bundle jre.bundle.
        PathBuf::from("jre.bundle")
            .join("Contents")
            .join("Home")
            .join("bin")
            .join("java")
    } else {
        PathBuf::from("bin").join("java")
    }
}

/// Dado el home de un runtime, devuelve la ruta al ejecutable java si existe.
/// Prueba primero el layout de Mojang y luego un layout plano `<home>/bin/java`
/// (util para runtimes de otros proveedores como Adoptium en fases futuras).
pub fn java_executable_in(home: &Path) -> Option<PathBuf> {
    let candidate = home.join(java_bin_relative());
    if candidate.exists() {
        return Some(candidate);
    }
    let flat = if cfg!(target_os = "windows") {
        home.join("bin").join("java.exe")
    } else {
        home.join("bin").join("java")
    };
    if flat.exists() {
        return Some(flat);
    }
    None
}