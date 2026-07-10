//! Superficie IPC del sistema de instancias. Rust es la fuente de verdad:
//! cada mutacion persiste en disco y actualiza el indice en memoria.

use crate::error::{AetherError, Result};
use crate::instance::{store, Instance, InstanceSummary, Loader};
use crate::AppState;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub name: String,
    pub mc_version: String,
    pub loader: Loader,
    pub loader_version: Option<String>,
    pub ram_mb: Option<u32>,
    pub icon: Option<String>,
    pub java_path: Option<String>,
}

/// Parche parcial. Los Option<Option<T>> permiten distinguir "no tocar"
/// (None) de "poner a vacio" (Some(None)).
#[derive(Debug, Deserialize)]
pub struct UpdatePatch {
    pub name: Option<String>,
    pub mc_version: Option<String>,
    pub loader: Option<Loader>,
    pub loader_version: Option<Option<String>>,
    pub ram_mb: Option<u32>,
    pub icon: Option<Option<String>>,
    pub java_path: Option<Option<String>>,
    pub favorite: Option<bool>,
}

#[tauri::command]
pub async fn list_instances(state: State<'_, AppState>) -> Result<Vec<InstanceSummary>> {
    Ok(state.index.lock().unwrap().clone())
}

#[tauri::command]
pub async fn get_instance(id: String) -> Result<Instance> {
    store::load_instance(&id)
}

#[tauri::command]
pub async fn create_instance(state: State<'_, AppState>, input: CreateInput) -> Result<Instance> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AetherError::InvalidState("el nombre no puede estar vacio".into()));
    }
    let id = Uuid::new_v4().simple().to_string();
    let dir = store::instances_dir()?.join(&id);
    let inst = Instance {
        id: id.clone(),
        name,
        mc_version: input.mc_version,
        loader: input.loader,
        loader_version: input.loader_version,
        ram_mb: input.ram_mb.unwrap_or(4096),
        path: dir.to_string_lossy().to_string(),
        icon: input.icon,
        java_path: input.java_path,
        favorite: false,
        created_at: now(),
        last_played: None,
        playtime_secs: 0,
        mod_count: 0,
        status: Default::default(),
        settings: Default::default(),
        modpack: Default::default(),
        java_major: None,
        installed_at: None,
        total_size_bytes: 0,
        last_error: None,
        installation: Default::default(),
    };
    store::save_instance(&inst)?;
    let mut index = state.index.lock().unwrap();
    index.push((&inst).into());
    store::save_index(&index)?;
    Ok(inst)
}

#[tauri::command]
pub async fn update_instance(
    state: State<'_, AppState>,
    id: String,
    patch: UpdatePatch,
) -> Result<Instance> {
    let mut inst = store::load_instance(&id)?;
    if let Some(v) = patch.name {
        let t = v.trim().to_string();
        if t.is_empty() {
            return Err(AetherError::InvalidState("el nombre no puede estar vacio".into()));
        }
        inst.name = t;
    }
    if let Some(v) = patch.mc_version {
        inst.mc_version = v;
    }
    if let Some(v) = patch.loader {
        inst.loader = v;
    }
    if let Some(v) = patch.loader_version {
        inst.loader_version = v;
    }
    if let Some(v) = patch.ram_mb {
        inst.ram_mb = v;
    }
    if let Some(v) = patch.icon {
        inst.icon = v;
    }
    if let Some(v) = patch.java_path {
        inst.java_path = v;
    }
    if let Some(v) = patch.favorite {
        inst.favorite = v;
    }
    store::save_instance(&inst)?;
    let mut index = state.index.lock().unwrap();
    if let Some(s) = index.iter_mut().find(|s| s.id == id) {
        *s = (&inst).into();
    }
    store::save_index(&index)?;
    Ok(inst)
}

#[tauri::command]
pub async fn delete_instance(state: State<'_, AppState>, id: String) -> Result<()> {
    store::delete_instance_dir(&id)?;
    let mut index = state.index.lock().unwrap();
    index.retain(|s| s.id != id);
    store::save_index(&index)?;
    Ok(())
}

#[tauri::command]
pub async fn duplicate_instance(state: State<'_, AppState>, id: String) -> Result<Instance> {
    let src = store::load_instance(&id)?;
    let new_id = Uuid::new_v4().simple().to_string();
    let dir = store::instances_dir()?.join(&new_id);
    let mut copy = src.clone();
    copy.id = new_id;
    copy.name = format!("{} (copia)", src.name);
    copy.path = dir.to_string_lossy().to_string();
    copy.created_at = now();
    copy.last_played = None;
    copy.playtime_secs = 0;
    copy.favorite = false;
    store::save_instance(&copy)?;
    let mut index = state.index.lock().unwrap();
    index.push((&copy).into());
    store::save_index(&index)?;
    Ok(copy)
}

/// Preparado para una fase futura. Aun no implementado a proposito.
#[tauri::command]
pub async fn import_instance(_path: String) -> Result<Instance> {
    Err(AetherError::NotImplemented(
        "la importacion de instancias llegara en una fase posterior".into(),
    ))
}
