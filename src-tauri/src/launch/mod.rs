//! Preparacion del lanzamiento de Minecraft Vanilla.
//!
//! Deja TODO listo para arrancar la JVM: detecta la instalacion, verifica el
//! client.jar, carga el version.json cacheado, arma el classpath (respetando
//! las reglas de librerias), resuelve el Java a usar (configurado o del
//! sistema) y construye los argumentos de JVM y de juego con todas las
//! variables sustituidas.
//!
//! NO ejecuta Java: el unico paso siguiente sera construir un Command a partir
//! de `LaunchPlan::command` y hacer spawn.

#![allow(dead_code)]

use crate::error::{AetherError, Result};
use crate::instance::store;
use crate::instance::Instance;
use crate::minecraft::version::{rules_allow, ArgElement, ArgValue, VersionDetail};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Separador de classpath segun el SO (Windows usa ';', el resto ':').
fn classpath_separator() -> char {
    if cfg!(target_os = "windows") { ';' } else { ':' }
}

/// Nombre del binario de Java segun el SO.
fn java_bin_name() -> &'static str {
    if cfg!(target_os = "windows") { "java.exe" } else { "java" }
}

/// Rutas resueltas de una instalacion concreta.
#[derive(Debug, Clone)]
pub struct InstalledPaths {
    pub data_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub natives_dir: PathBuf,
    pub game_dir: PathBuf,
    pub version_json: PathBuf,
    pub client_jar: PathBuf,
}

/// Plan de lanzamiento completo. `command` es el argv final listo para
/// Command::new (command[0] = binario de Java), SIN ejecutarse todavia.
#[derive(Debug, Clone)]
pub struct LaunchPlan {
    pub java: PathBuf,
    pub jvm_args: Vec<String>,
    pub main_class: String,
    pub game_args: Vec<String>,
    pub classpath: Vec<PathBuf>,
    pub game_dir: PathBuf,
    pub command: Vec<String>,
}

/// Detecta la instalacion: resuelve todas las rutas (misma convencion que el
/// instalador) y comprueba que los artefactos criticos existan en disco.
pub fn detect_installation(instance: &Instance) -> Result<InstalledPaths> {
    let data_dir = store::data_dir()?;
    let versions_dir = data_dir.join("versions").join(&instance.mc_version);
    let game_dir = Path::new(&instance.path).join("minecraft");

    let paths = InstalledPaths {
        libraries_dir: data_dir.join("libraries"),
        assets_dir: data_dir.join("assets"),
        natives_dir: game_dir.join("natives"),
        version_json: versions_dir.join(format!("{}.json", instance.mc_version)),
        client_jar: versions_dir.join(format!("{}.jar", instance.mc_version)),
        game_dir,
        data_dir,
    };

    if !paths.version_json.exists() {
        return Err(AetherError::NotFound(format!(
            "version.json de {} (la instancia no esta instalada)",
            instance.mc_version
        )));
    }
    if !paths.client_jar.exists() {
        return Err(AetherError::NotFound(format!(
            "client.jar de {}",
            instance.mc_version
        )));
    }
    Ok(paths)
}

/// Calcula el SHA1 de un archivo (lectura sincrona).
fn sha1_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    let mut h = Sha1::new();
    h.update(&bytes);
    Ok(hex::encode(h.finalize()))
}

/// Verifica el client.jar contra el SHA1 esperado del version.json.
pub fn verify_client_jar(client_jar: &Path, expected_sha1: &str) -> Result<()> {
    let actual = sha1_file(client_jar)?;
    if actual != expected_sha1 {
        return Err(AetherError::HashMismatch {
            path: client_jar.to_string_lossy().to_string(),
            expected: expected_sha1.to_string(),
            actual,
        });
    }
    Ok(())
}

/// Carga el version.json ya descargado en disco (sin red).
pub fn load_version_detail(version_json: &Path) -> Result<VersionDetail> {
    let raw = std::fs::read_to_string(version_json)?;
    Ok(serde_json::from_str(&raw)?)
}

/// Convierte un nombre maven (grupo:artefacto:version) a ruta relativa .jar.
/// Debe coincidir con la convencion usada por el instalador.
fn maven_to_path(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return name.replace(':', "/");
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    format!(
        "{}/{}/{}/{}-{}.jar",
        group.replace('.', "/"),
        artifact,
        version,
        artifact,
        version
    )
}

/// Arma el classpath: librerias permitidas (respetando reglas) + client.jar.
pub fn build_classpath(
    detail: &VersionDetail,
    libraries_dir: &Path,
    client_jar: &Path,
) -> Vec<PathBuf> {
    let mut cp: Vec<PathBuf> = Vec::new();
    for lib in &detail.libraries {
        if !lib.allowed() {
            continue;
        }
        if let Some(dl) = &lib.downloads {
            if let Some(art) = &dl.artifact {
                let rel = art.path.clone().unwrap_or_else(|| maven_to_path(&lib.name));
                cp.push(libraries_dir.join(rel));
            }
        }
    }
    cp.push(client_jar.to_path_buf());
    cp
}

/// Resuelve el binario de Java: primero el configurado en la instancia,
/// luego JAVA_HOME, y como ultimo recurso "java" del PATH.
pub fn resolve_java(instance: &Instance) -> PathBuf {
    if let Some(configured) = &instance.java_path {
        let p = PathBuf::from(configured);
        if p.exists() {
            return p;
        }
    }
    if let Ok(home) = std::env::var("JAVA_HOME") {
        if !home.is_empty() {
            let p = PathBuf::from(home).join("bin").join(java_bin_name());
            if p.exists() {
                return p;
            }
        }
    }
    PathBuf::from(java_bin_name())
}

/// Sustituye todas las variables ${...} conocidas en una plantilla.
fn substitute(template: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        let needle = format!("${{{}}}", key);
        if out.contains(&needle) {
            out = out.replace(&needle, value);
        }
    }
    out
}

/// Aplana una lista de argumentos modernos aplicando reglas y sustituyendo
/// variables. Los condicionales cuyas reglas no aplican se descartan.
fn collect_args(elements: &[ArgElement], vars: &HashMap<&str, String>) -> Vec<String> {
    let mut out = Vec::new();
    for el in elements {
        match el {
            ArgElement::Simple(s) => out.push(substitute(s, vars)),
            ArgElement::Conditional { rules, value } => {
                if !rules_allow(rules) {
                    continue;
                }
                match value {
                    ArgValue::Single(s) => out.push(substitute(s, vars)),
                    ArgValue::Many(list) => {
                        for s in list {
                            out.push(substitute(s, vars));
                        }
                    }
                }
            }
        }
    }
    out
}

/// Prepara el lanzamiento completo de Minecraft Vanilla. Deja todo listo para
/// que el unico paso siguiente sea ejecutar `plan.command`.
pub fn prepare_launch(instance: &Instance) -> Result<LaunchPlan> {
    // 1. Deteccion de instalacion (rutas + existencia de artefactos).
    let paths = detect_installation(instance)?;

    // 2. Carga del version.json cacheado.
    let detail = load_version_detail(&paths.version_json)?;

    // 3. Verificacion del client.jar contra su SHA1.
    verify_client_jar(&paths.client_jar, &detail.downloads.client.sha1)?;

    // 4. Classpath (librerias permitidas + client.jar).
    let classpath = build_classpath(&detail, &paths.libraries_dir, &paths.client_jar);
    let sep = classpath_separator().to_string();
    let classpath_str = classpath
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(&sep);

    // 5. Resolucion del Java (configurado -> JAVA_HOME -> PATH).
    let java = resolve_java(instance);

    // 6. Tabla de variables para la sustitucion.
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("natives_directory", paths.natives_dir.to_string_lossy().to_string());
    vars.insert("launcher_name", "AetherLauncher".into());
    vars.insert("launcher_version", env!("CARGO_PKG_VERSION").into());
    vars.insert("classpath", classpath_str);
    vars.insert("classpath_separator", sep);
    vars.insert("library_directory", paths.libraries_dir.to_string_lossy().to_string());
    vars.insert("version_name", instance.mc_version.clone());
    vars.insert("version_type", "release".into());
    vars.insert("game_directory", paths.game_dir.to_string_lossy().to_string());
    vars.insert("assets_root", paths.assets_dir.to_string_lossy().to_string());
    vars.insert("game_assets", paths.assets_dir.to_string_lossy().to_string());
    vars.insert("assets_index_name", detail.assets.clone());
    // Placeholders de autenticacion (se completaran en la fase de login).
    vars.insert("auth_player_name", "Player".into());
    vars.insert("auth_uuid", "00000000000000000000000000000000".into());
    vars.insert("auth_access_token", "0".into());
    vars.insert("auth_session", "0".into());
    vars.insert("auth_xuid", "0".into());
    vars.insert("clientid", "0".into());
    vars.insert("user_type", "msa".into());
    vars.insert("user_properties", "{}".into());

    // 7. Argumentos de JVM y de juego (formato moderno o legacy).
    let (jvm_args, game_args) = match &detail.arguments {
        Some(arguments) => (
            collect_args(&arguments.jvm, &vars),
            collect_args(&arguments.game, &vars),
        ),
        None => {
            // Legacy (< 1.13): minecraftArguments + JVM por defecto.
            let jvm = vec![
                format!("-Djava.library.path={}", paths.natives_dir.to_string_lossy()),
                "-cp".to_string(),
                vars.get("classpath").cloned().unwrap_or_default(),
            ];
            let game = detail
                .minecraft_arguments
                .as_deref()
                .unwrap_or("")
                .split_whitespace()
                .map(|s| substitute(s, &vars))
                .collect();
            (jvm, game)
        }
    };

    // 8. argv final: java + jvm + mainClass + game (SIN ejecutar todavia).
    let mut command = Vec::with_capacity(jvm_args.len() + game_args.len() + 2);
    command.push(java.to_string_lossy().to_string());
    command.extend(jvm_args.iter().cloned());
    command.push(detail.main_class.clone());
    command.extend(game_args.iter().cloned());

    Ok(LaunchPlan {
        java,
        jvm_args,
        main_class: detail.main_class.clone(),
        game_args,
        classpath,
        game_dir: paths.game_dir,
        command,
    })
}