//! Persistencia en disco. Indice en JSON, detalle por instancia en TOML.
//! Toda escritura es atomica (archivo temporal + rename) para no corromper
//! datos si el proceso muere a mitad de guardado.

use crate::error::{AetherError, Result};
use crate::instance::{Instance, InstanceSummary};
use std::fs;
use std::path::{Path, PathBuf};

pub fn data_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "aether", "launcher")
        .ok_or_else(|| AetherError::InvalidState("no se pudo resolver el directorio de datos".into()))?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn instances_dir() -> Result<PathBuf> {
    let d = data_dir()?.join("instances");
    fs::create_dir_all(&d)?;
    Ok(d)
}

fn index_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("instances.json"))
}

fn instance_toml(id: &str) -> Result<PathBuf> {
    Ok(instances_dir()?.join(id).join("instance.toml"))
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn load_index() -> Result<Vec<InstanceSummary>> {
    let p = index_path()?;
    if !p.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&p)?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_index(list: &[InstanceSummary]) -> Result<()> {
    let json = serde_json::to_vec_pretty(list)?;
    atomic_write(&index_path()?, &json)
}

pub fn load_instance(id: &str) -> Result<Instance> {
    let p = instance_toml(id)?;
    if !p.exists() {
        return Err(AetherError::NotFound(format!("instancia {id}")));
    }
    Ok(toml::from_str(&fs::read_to_string(&p)?)?)
}

pub fn save_instance(inst: &Instance) -> Result<()> {
    let dir = instances_dir()?.join(&inst.id);
    // El .minecraft real vivira aqui: mods/, saves/, shaderpacks/, etc. (fases futuras).
    fs::create_dir_all(dir.join("minecraft"))?;
    let toml_str = toml::to_string_pretty(inst)?;
    atomic_write(&dir.join("instance.toml"), toml_str.as_bytes())
}

pub fn delete_instance_dir(id: &str) -> Result<()> {
    let dir = instances_dir()?.join(id);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}
