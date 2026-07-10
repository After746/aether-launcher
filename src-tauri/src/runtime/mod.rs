//! Runtime Manager: administra los runtimes de Java que necesita cada
//! version de Minecraft, sin depender de JAVA_HOME, PATH ni instalaciones
//! manuales del usuario. Fase 1: cimientos + deteccion del Java requerido.
//!
//! Se apoya en la arquitectura existente (store::data_dir, minecraft::*,
//! cliente HTTP compartido) y NO reimplementa instalacion, launch ni natives.

pub mod commands;
pub mod detect;
pub mod install; 
pub mod manifest;
pub mod paths;
pub mod provide;

use serde::{Deserialize, Serialize};

/// Java requerido por una version, resuelto por el detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredJava {
    /// Version mayor de Java necesaria (p.ej. 8, 16, 17, 21).
    pub major: u32,
    /// De donde salio el dato.
    pub source: RequiredJavaSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredJavaSource {
    /// El campo `javaVersion.majorVersion` estaba presente en el version.json.
    VersionJson,
    /// El version.json no declara Java (versiones antiguas): fallback a Java 8.
    Fallback,
}

/// Un runtime de Java administrado por Aether, materializado en disco.
/// En Fase 1 solo se DESCUBRE (no se descarga todavia).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedRuntime {
    /// Version mayor de Java (8, 17, 21, ...).
    pub major: u32,
    /// Ruta absoluta al ejecutable de Java (bin/java[.exe]).
    pub java_path: String,
    /// Ruta absoluta a la raiz del runtime (la carpeta que contiene bin/).
    pub home: String,
}

/// Resultado de resolver que Java necesita una instancia y si ya lo tenemos.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeResolution {
    /// Java que la version necesita.
    pub required: RequiredJava,
    /// Runtime administrado ya disponible para ese major, si existe.
    /// En Fase 1 casi siempre sera `None` (aun no descargamos nada).
    pub managed: Option<ManagedRuntime>,
    /// true si hay un runtime administrado listo para usar.
    pub ready: bool,
    
}
/// Eventos de progreso emitidos durante la descarga de un runtime.
#[derive(Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeEvent {
    Started {
        major: u32,
        version: String,
        total_files: u64,
        total_bytes: u64,
    },
    Phase {
        phase: String,
    },
    Progress {
        files_done: u64,
        total_files: u64,
        bytes_done: u64,
        total_bytes: u64,
        current_file: String,
        speed_bps: u64,
    },
    Done {
        major: u32,
        java_path: String,
    },
    Failed {
        message: String,
    },
}
