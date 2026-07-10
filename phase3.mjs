// ============================================================================
//  Aether Launcher - Fase 3: Motor de descargas e instalacion (Vanilla)
//  Uso:  node phase3.mjs   (ejecutar DENTRO de aether-launcher)
//  UTF-8 real, sin escapes ni PowerShell.
// ============================================================================
import { writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';

const root = process.cwd();
if (!existsSync(join(root, 'package.json'))) {
  console.error("ERROR: ejecuta este script dentro de la carpeta 'aether-launcher'.");
  process.exit(1);
}
function write(path, content) {
  const full = join(root, path);
  mkdirSync(dirname(full), { recursive: true });
  writeFileSync(full, content, { encoding: 'utf8' });
  console.log('  ~ ' + path);
}
console.log('\nAplicando Fase 3 en: ' + root + '\n');

// ===========================================================================
//  BACKEND
// ===========================================================================

write('src-tauri/Cargo.toml', `[package]
name = "aether-launcher"
version = "0.1.0"
description = "El launcher de Minecraft mas moderno, rapido y elegante."
edition = "2021"
rust-version = "1.77"

[lib]
name = "aether_launcher_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
directories = "5"
tokio = { version = "1", features = ["full"] }
tokio-util = "0.7"
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false }
futures = "0.3"
sha1 = "0.10"
hex = "0.4"

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
`);

write('src-tauri/src/minecraft/manifest.rs', `//! Version manifest oficial de Mojang: indice global de versiones y
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
`);

write('src-tauri/src/minecraft/version.rs', `//! JSON detallado de una version: librerias (con reglas por SO), assets,
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
`);

write('src-tauri/src/minecraft/mod.rs', `//! Nucleo de Minecraft: manifest, parseo de version y assets.
pub mod manifest;
pub mod version;
`);

write('src-tauri/src/download/mod.rs', `//! Pool de descargas concurrente sobre Tokio. Streaming a disco,
//! verificacion SHA1, skip-if-valid (reanudable) y agregacion de progreso.

use crate::error::{AetherError, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

const MAX_CONCURRENT: usize = 12;

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
    pub sha1: Option<String>,
    pub size: u64,
}

/// Contadores compartidos para reportar progreso agregado.
#[derive(Default)]
pub struct Progress {
    pub files_done: AtomicU64,
    pub bytes_done: AtomicU64,
    pub total_files: AtomicU64,
    pub total_bytes: AtomicU64,
}

async fn sha1_of(path: &PathBuf) -> Option<String> {
    let bytes = tokio::fs::read(path).await.ok()?;
    let mut h = Sha1::new();
    h.update(&bytes);
    Some(hex::encode(h.finalize()))
}

/// Descarga un archivo con streaming. Si ya existe y el hash coincide, se salta.
async fn fetch_one(
    client: reqwest::Client,
    task: DownloadTask,
    progress: Arc<Progress>,
    current: Arc<tokio::sync::Mutex<String>>,
    cancel: CancellationToken,
) -> Result<()> {
    if cancel.is_cancelled() {
        return Err(AetherError::InvalidState("cancelado".into()));
    }

    // Skip-if-valid: reanudable e idempotente.
    if task.dest.exists() {
        if let Some(expected) = &task.sha1 {
            if let Some(actual) = sha1_of(&task.dest).await {
                if &actual == expected {
                    progress.files_done.fetch_add(1, Ordering::Relaxed);
                    progress.bytes_done.fetch_add(task.size, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }
    }

    if let Some(parent) = task.dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    {
        let name = task
            .dest
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        *current.lock().await = name;
    }

    let tmp = task.dest.with_extension("part");
    let mut resp = client.get(&task.url).send().await?.error_for_status()?;
    let mut file = tokio::fs::File::create(&tmp).await?;
    let mut hasher = Sha1::new();

    while let Some(chunk) = resp.chunk().await? {
        if cancel.is_cancelled() {
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err(AetherError::InvalidState("cancelado".into()));
        }
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        progress.bytes_done.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    }
    file.flush().await?;
    drop(file);

    if let Some(expected) = &task.sha1 {
        let actual = hex::encode(hasher.finalize());
        if &actual != expected {
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err(AetherError::HashMismatch {
                expected: expected.clone(),
                actual,
            });
        }
    }

    tokio::fs::rename(&tmp, &task.dest).await?;
    progress.files_done.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

/// Ejecuta un lote de descargas con concurrencia limitada.
pub async fn run_batch(
    client: &reqwest::Client,
    tasks: Vec<DownloadTask>,
    progress: Arc<Progress>,
    current: Arc<tokio::sync::Mutex<String>>,
    cancel: CancellationToken,
) -> Result<()> {
    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut futs = FuturesUnordered::new();

    for task in tasks {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let progress = progress.clone();
        let current = current.clone();
        let cancel = cancel.clone();
        futs.push(tokio::spawn(async move {
            let _permit = permit;
            fetch_one(client, task, progress, current, cancel).await
        }));
    }

    while let Some(joined) = futs.next().await {
        match joined {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(AetherError::InvalidState(format!("tarea abortada: {e}"))),
        }
    }
    Ok(())
}
`);

write('src-tauri/src/install/mod.rs', `//! Trait Installer: contrato comun para todos los loaders. Vanilla se
//! implementa completo; Fabric/Forge/NeoForge quedan preparados.

pub mod commands;
pub mod vanilla;

use crate::error::{AetherError, Result};
use crate::instance::{Instance, Loader};
use serde::Serialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstallEvent {
    Started { total_files: u64, total_bytes: u64 },
    Phase { phase: String },
    Progress {
        files_done: u64,
        total_files: u64,
        bytes_done: u64,
        total_bytes: u64,
        current_file: String,
        speed_bps: u64,
    },
    Done,
    Cancelled,
    Failed { message: String },
}

pub trait Installer {
    async fn install(
        &self,
        client: &reqwest::Client,
        instance: &Instance,
        on_event: &Channel<InstallEvent>,
        cancel: CancellationToken,
    ) -> Result<u64>; // devuelve bytes totales instalados
}

/// Enruta al instalador correcto segun el loader de la instancia.
pub async fn install_for(
    client: &reqwest::Client,
    instance: &Instance,
    on_event: &Channel<InstallEvent>,
    cancel: CancellationToken,
) -> Result<u64> {
    match instance.loader {
        Loader::Vanilla => {
            vanilla::VanillaInstaller
                .install(client, instance, on_event, cancel)
                .await
        }
        Loader::Fabric => Err(AetherError::NotImplemented(
            "el instalador de Fabric llegara en una fase posterior".into(),
        )),
        Loader::Forge => Err(AetherError::NotImplemented(
            "el instalador de Forge llegara en una fase posterior".into(),
        )),
        Loader::NeoForge => Err(AetherError::NotImplemented(
            "el instalador de NeoForge llegara en una fase posterior".into(),
        )),
    }
}
`);

write('src-tauri/src/install/vanilla.rs', `//! Instalador Vanilla completo. Orden por capas:
//! manifest -> version JSON -> libraries -> client jar -> assets -> natives.

use super::{InstallEvent, Installer};
use crate::download::{run_batch, DownloadTask, Progress};
use crate::error::Result;
use crate::instance::store;
use crate::instance::Instance;
use crate::minecraft::manifest::VersionManifest;
use crate::minecraft::version::{AssetIndex, Library, VersionDetail};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

pub struct VanillaInstaller;

/// Convierte un nombre maven (grupo:artefacto:version) a ruta relativa.
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

impl Installer for VanillaInstaller {
    async fn install(
        &self,
        client: &reqwest::Client,
        instance: &Instance,
        on_event: &Channel<InstallEvent>,
        cancel: CancellationToken,
    ) -> Result<u64> {
        let data = store::data_dir()?;
        let libraries_dir = data.join("libraries");
        let assets_dir = data.join("assets");
        let versions_dir = data.join("versions").join(&instance.mc_version);
        let natives_dir = std::path::Path::new(&instance.path).join("minecraft").join("natives");

        // --- Capa 1: manifest ---
        let _ = on_event.send(InstallEvent::Phase { phase: "manifest".into() });
        let manifest = VersionManifest::fetch(client).await?;
        let entry = manifest.find(&instance.mc_version)?;

        // --- Capa 2: version JSON (cacheado) ---
        let _ = on_event.send(InstallEvent::Phase { phase: "version_json".into() });
        let detail: VersionDetail = client
            .get(&entry.url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        tokio::fs::create_dir_all(&versions_dir).await?;
        let vjson = client.get(&entry.url).send().await?.text().await?;
        tokio::fs::write(versions_dir.join(format!("{}.json", instance.mc_version)), vjson).await?;

        // --- Asset index ---
        let asset_index: AssetIndex = client
            .get(&detail.asset_index.url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        tokio::fs::create_dir_all(assets_dir.join("indexes")).await?;
        let ai_raw = client.get(&detail.asset_index.url).send().await?.text().await?;
        tokio::fs::write(
            assets_dir.join("indexes").join(format!("{}.json", detail.asset_index.id)),
            ai_raw,
        )
        .await?;

        // --- Construir el lote completo de tareas ---
        let mut tasks: Vec<DownloadTask> = Vec::new();

        // Libraries (filtradas por SO) + natives.
        for lib in &detail.libraries {
            if !lib.allowed() {
                continue;
            }
            if let Some(dl) = &lib.downloads {
                if let Some(art) = &dl.artifact {
                    let rel = art.path.clone().unwrap_or_else(|| maven_to_path(&lib.name));
                    tasks.push(DownloadTask {
                        url: art.url.clone(),
                        dest: libraries_dir.join(&rel),
                        sha1: Some(art.sha1.clone()),
                        size: art.size,
                    });
                }
                // Natives: se guardan por instancia (se extraeran en Fase 4).
                if let Some(classifier) = lib.native_classifier() {
                    if let Some(classifiers) = &dl.classifiers {
                        if let Some(art) = classifiers.get(&classifier) {
                            let fname = art
                                .path
                                .clone()
                                .unwrap_or_else(|| format!("{}.jar", classifier));
                            let leaf = std::path::Path::new(&fname)
                                .file_name()
                                .map(|n| n.to_owned())
                                .unwrap_or_else(|| std::ffi::OsString::from("native.jar"));
                            tasks.push(DownloadTask {
                                url: art.url.clone(),
                                dest: natives_dir.join(leaf),
                                sha1: Some(art.sha1.clone()),
                                size: art.size,
                            });
                        }
                    }
                }
            }
        }

        // Client jar.
        tasks.push(DownloadTask {
            url: detail.downloads.client.url.clone(),
            dest: versions_dir.join(format!("{}.jar", instance.mc_version)),
            sha1: Some(detail.downloads.client.sha1.clone()),
            size: detail.downloads.client.size,
        });

        // Assets (caché global por hash).
        for obj in asset_index.objects.values() {
            let sub = &obj.hash[0..2];
            tasks.push(DownloadTask {
                url: format!(
                    "https://resources.download.minecraft.net/{}/{}",
                    sub, obj.hash
                ),
                dest: assets_dir.join("objects").join(sub).join(&obj.hash),
                sha1: Some(obj.hash.clone()),
                size: obj.size,
            });
        }

        // --- Progreso ---
        let progress = Arc::new(Progress::default());
        let total_files = tasks.len() as u64;
        let total_bytes: u64 = tasks.iter().map(|t| t.size).sum();
        progress.total_files.store(total_files, Ordering::Relaxed);
        progress.total_bytes.store(total_bytes, Ordering::Relaxed);
        let _ = on_event.send(InstallEvent::Started { total_files, total_bytes });
        let _ = on_event.send(InstallEvent::Phase { phase: "libraries".into() });

        let current = Arc::new(tokio::sync::Mutex::new(String::new()));

        // Emisor de progreso: ~10 Hz, calcula velocidad con ventana simple.
        let reporter = {
            let progress = progress.clone();
            let current = current.clone();
            let on_event = on_event.clone();
            let cancel = cancel.clone();
            tokio::spawn(async move {
                let start = Instant::now();
                let mut last_bytes = 0u64;
                let mut last_t = start;
                loop {
                    if cancel.is_cancelled() {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    let done = progress.files_done.load(Ordering::Relaxed);
                    let bytes = progress.bytes_done.load(Ordering::Relaxed);
                    let now = Instant::now();
                    let dt = now.duration_since(last_t).as_secs_f64().max(0.001);
                    let speed = ((bytes.saturating_sub(last_bytes)) as f64 / dt) as u64;
                    last_bytes = bytes;
                    last_t = now;
                    let cur = current.lock().await.clone();
                    let _ = on_event.send(InstallEvent::Progress {
                        files_done: done,
                        total_files,
                        bytes_done: bytes,
                        total_bytes,
                        current_file: cur,
                        speed_bps: speed,
                    });
                    if done >= total_files {
                        break;
                    }
                }
            })
        };

        let _ = on_event.send(InstallEvent::Phase { phase: "assets".into() });
        let result = run_batch(client, tasks, progress.clone(), current.clone(), cancel.clone()).await;
        reporter.abort();

        result?;
        Ok(total_bytes)
    }
}
`);

write('src-tauri/src/install/commands.rs', `//! Comandos IPC de instalacion. Gestiona el ciclo de estados y la
//! cancelacion mediante CancellationToken por instancia.

use crate::error::{AetherError, Result};
use crate::install::{install_for, InstallEvent};
use crate::instance::{store, InstallStatus};
use crate::AppState;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

/// Registro global de instalaciones en curso (id -> token de cancelacion).
#[derive(Default)]
pub struct InstallRegistry {
    pub active: Mutex<HashMap<String, CancellationToken>>,
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn set_status(state: &AppState, id: &str, status: InstallStatus) {
    let mut index = state.index.lock().unwrap();
    if let Some(s) = index.iter_mut().find(|s| s.id == id) {
        s.status = status;
    }
    let _ = store::save_index(&index);
}

#[tauri::command]
pub async fn install_instance(
    state: State<'_, AppState>,
    registry: State<'_, InstallRegistry>,
    id: String,
    on_event: Channel<InstallEvent>,
) -> Result<()> {
    let mut inst = store::load_instance(&id)?;

    let cancel = CancellationToken::new();
    registry.active.lock().unwrap().insert(id.clone(), cancel.clone());

    inst.status = InstallStatus::Installing;
    inst.installation.current_phase = "starting".into();
    inst.installation.progress = 0.0;
    inst.last_error = None;
    let _ = store::save_instance(&inst);
    set_status(&state, &id, InstallStatus::Installing);

    let result = install_for(&state.http, &inst, &on_event, cancel.clone()).await;
    registry.active.lock().unwrap().remove(&id);

    match result {
        Ok(bytes) => {
            inst.status = InstallStatus::Ready;
            inst.installed_at = Some(now());
            inst.total_size_bytes = bytes;
            inst.installation.current_phase = "done".into();
            inst.installation.progress = 1.0;
            let _ = store::save_instance(&inst);
            set_status(&state, &id, InstallStatus::Ready);
            let _ = on_event.send(InstallEvent::Done);
            Ok(())
        }
        Err(_) if cancel.is_cancelled() => {
            inst.status = InstallStatus::Cancelled;
            inst.installation.current_phase = "cancelled".into();
            let _ = store::save_instance(&inst);
            set_status(&state, &id, InstallStatus::Cancelled);
            let _ = on_event.send(InstallEvent::Cancelled);
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            inst.status = InstallStatus::Error;
            inst.last_error = Some(msg.clone());
            inst.installation.current_phase = "error".into();
            let _ = store::save_instance(&inst);
            set_status(&state, &id, InstallStatus::Error);
            let _ = on_event.send(InstallEvent::Failed { message: msg.clone() });
            Err(AetherError::InvalidState(msg))
        }
    }
}

#[tauri::command]
pub async fn cancel_install(registry: State<'_, InstallRegistry>, id: String) -> Result<()> {
    if let Some(token) = registry.active.lock().unwrap().get(&id) {
        token.cancel();
        Ok(())
    } else {
        Err(AetherError::NotFound(format!("no hay instalacion activa para {id}")))
    }
}

/// Lista de versiones de Minecraft para el selector de la UI.
#[tauri::command]
pub async fn list_mc_versions(
    state: State<'_, AppState>,
) -> Result<Vec<crate::minecraft::version::VersionOption>> {
    use crate::minecraft::manifest::VersionManifest;
    let manifest = VersionManifest::fetch(&state.http).await?;
    Ok(manifest
        .versions
        .into_iter()
        .map(|v| crate::minecraft::version::VersionOption {
            id: v.id,
            kind: v.kind,
            release_time: v.release_time,
        })
        .collect())
}
`);

write('src-tauri/src/lib.rs', `//! Aether Launcher - nucleo de la aplicacion de escritorio (Tauri 2).

mod download;
mod error;
mod install;
mod instance;
mod minecraft;

use std::sync::Mutex;

/// Estado global. Indice en memoria + cliente HTTP reutilizado (pool keep-alive).
pub struct AppState {
    pub index: Mutex<Vec<instance::InstanceSummary>>,
    pub http: reqwest::Client,
}

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let index = instance::store::load_index().unwrap_or_default();
    let http = reqwest::Client::builder()
        .user_agent(concat!("AetherLauncher/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("no se pudo construir el cliente HTTP");

    tauri::Builder::default()
        .manage(AppState { index: Mutex::new(index), http })
        .manage(install::commands::InstallRegistry::default())
        .invoke_handler(tauri::generate_handler![
            app_version,
            instance::commands::list_instances,
            instance::commands::get_instance,
            instance::commands::create_instance,
            instance::commands::update_instance,
            instance::commands::delete_instance,
            instance::commands::duplicate_instance,
            instance::commands::import_instance,
            install::commands::install_instance,
            install::commands::cancel_install,
            install::commands::list_mc_versions
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar Aether Launcher");
}
`);

write('src-tauri/src/instance/mod.rs', `//! Modelo de dominio de una instancia de Minecraft.
//! Cada instancia es una entidad autocontenida y persistida como TOML.

pub mod commands;
pub mod store;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
}
impl Default for Loader {
    fn default() -> Self {
        Loader::Vanilla
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStatus {
    Created,
    Installing,
    Cancelled,
    Ready,
    Error,
    Corrupt,
}
impl Default for InstallStatus {
    fn default() -> Self {
        InstallStatus::Created
    }
}

/// Preparado para una fase futura. Aun no se consume funcionalmente.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceSettings {
    #[serde(default)]
    pub resolution_width: Option<u32>,
    #[serde(default)]
    pub resolution_height: Option<u32>,
    #[serde(default)]
    pub fullscreen: bool,
}

/// Preparado para Modrinth / CurseForge. Aun sin implementar.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Modpack {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
}

/// Estado persistido de instalacion, para recuperar si el launcher se cierra.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Installation {
    #[serde(default)]
    pub current_phase: String,
    #[serde(default)]
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    #[serde(default)]
    pub loader: Loader,
    #[serde(default)]
    pub loader_version: Option<String>,
    pub ram_mb: u32,
    pub path: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub java_path: Option<String>,
    /// Major de Java recomendado por la version (lo rellena Fase 3, lo usa Fase 4).
    #[serde(default)]
    pub java_major: Option<u32>,
    #[serde(default)]
    pub favorite: bool,
    pub created_at: i64,
    #[serde(default)]
    pub last_played: Option<i64>,
    #[serde(default)]
    pub playtime_secs: u64,
    #[serde(default)]
    pub mod_count: u32,
    #[serde(default)]
    pub status: InstallStatus,
    #[serde(default)]
    pub installed_at: Option<i64>,
    #[serde(default)]
    pub total_size_bytes: u64,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub settings: InstanceSettings,
    #[serde(default)]
    pub modpack: Modpack,
    #[serde(default)]
    pub installation: Installation,
}

/// Resumen ligero para el indice (instances.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSummary {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub loader: Loader,
    pub ram_mb: u32,
    pub icon: Option<String>,
    pub last_played: Option<i64>,
    pub playtime_secs: u64,
    pub mod_count: u32,
    pub favorite: bool,
    pub status: InstallStatus,
    #[serde(default)]
    pub total_size_bytes: u64,
}

impl From<&Instance> for InstanceSummary {
    fn from(i: &Instance) -> Self {
        Self {
            id: i.id.clone(),
            name: i.name.clone(),
            mc_version: i.mc_version.clone(),
            loader: i.loader,
            ram_mb: i.ram_mb,
            icon: i.icon.clone(),
            last_played: i.last_played,
            playtime_secs: i.playtime_secs,
            mod_count: i.mod_count,
            favorite: i.favorite,
            status: i.status,
            total_size_bytes: i.total_size_bytes,
        }
    }
}
`);

// ===========================================================================
//  FRONTEND
// ===========================================================================

write('src/lib/ipc/types.ts', `// Tipos espejo del modelo Rust. Sincronizar con src-tauri/src/instance/mod.rs.

export type Loader = 'vanilla' | 'fabric' | 'forge' | 'neoforge';
export type InstallStatus =
  | 'created' | 'installing' | 'cancelled' | 'ready' | 'error' | 'corrupt';

export interface InstanceSummary {
  id: string;
  name: string;
  mc_version: string;
  loader: Loader;
  ram_mb: number;
  icon: string | null;
  last_played: number | null;
  playtime_secs: number;
  mod_count: number;
  favorite: boolean;
  status: InstallStatus;
  total_size_bytes: number;
}

export interface Instance extends InstanceSummary {
  loader_version: string | null;
  path: string;
  java_path: string | null;
  java_major: number | null;
  created_at: number;
  installed_at: number | null;
  last_error: string | null;
}

export interface CreateInput {
  name: string;
  mc_version: string;
  loader: Loader;
  loader_version?: string | null;
  ram_mb?: number | null;
  icon?: string | null;
  java_path?: string | null;
}

export interface VersionOption {
  id: string;
  kind: string;
  release_time: string;
}

export type InstallEvent =
  | { kind: 'started'; total_files: number; total_bytes: number }
  | { kind: 'phase'; phase: string }
  | {
      kind: 'progress';
      files_done: number;
      total_files: number;
      bytes_done: number;
      total_bytes: number;
      current_file: string;
      speed_bps: number;
    }
  | { kind: 'done' }
  | { kind: 'cancelled' }
  | { kind: 'failed'; message: string };

export const LOADER_LABEL: Record<Loader, string> = {
  vanilla: 'Vanilla',
  fabric: 'Fabric',
  forge: 'Forge',
  neoforge: 'NeoForge',
};

export const STATUS_LABEL: Record<InstallStatus, string> = {
  created: 'Sin instalar',
  installing: 'Instalando',
  cancelled: 'Cancelada',
  ready: 'Lista',
  error: 'Error',
  corrupt: 'Dañada',
};

export const PHASE_LABEL: Record<string, string> = {
  starting: 'Preparando',
  manifest: 'Obteniendo versiones',
  version_json: 'Leyendo versión',
  libraries: 'Descargando librerías',
  client: 'Descargando cliente',
  assets: 'Descargando recursos',
  natives: 'Extrayendo nativos',
  done: 'Completado',
};
`);

write('src/lib/components/ui/Dropdown.svelte', `<script lang="ts">
  import Icon from './Icon.svelte';
  import { slide } from 'svelte/transition';

  export type Option = { value: string; label: string; hint?: string };

  let { value = $bindable(), options, placeholder = 'Selecciona…' }: {
    value: string;
    options: Option[];
    placeholder?: string;
  } = $props();

  let open = $state(false);
  let root: HTMLDivElement;

  const current = $derived(options.find((o) => o.value === value) ?? null);

  function choose(v: string) {
    value = v;
    open = false;
  }
  function onWindowClick(e: MouseEvent) {
    if (root && !root.contains(e.target as Node)) open = false;
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={onKey} />

<div class="dd" bind:this={root}>
  <button type="button" class="trigger" class:open onclick={() => (open = !open)}>
    <span class="value" class:placeholder={!current}>{current ? current.label : placeholder}</span>
    <span class="chev" class:open><Icon name="chevron" size={16} /></span>
  </button>

  {#if open}
    <div class="menu" transition:slide={{ duration: 160 }}>
      {#each options as opt (opt.value)}
        <button
          type="button"
          class="opt"
          class:active={opt.value === value}
          onclick={() => choose(opt.value)}>
          <span class="opt-label">{opt.label}</span>
          {#if opt.hint}<span class="opt-hint">{opt.hint}</span>{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dd { position: relative; width: 100%; }
  .trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 12px;
    font: inherit;
    color: var(--text);
    background: var(--panel);
    border: 1px solid var(--stroke);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: border-color var(--dur) var(--ease), background var(--dur) var(--ease);
  }
  .trigger:hover { background: var(--panel-strong); }
  .trigger.open { border-color: var(--accent); background: var(--panel-strong); }
  .value { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .value.placeholder { color: var(--muted); }
  .chev { color: var(--muted); transition: transform var(--dur) var(--ease); }
  .chev.open { transform: rotate(90deg); }

  .menu {
    position: absolute;
    z-index: 20;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    max-height: 240px;
    overflow-y: auto;
    padding: 6px;
    border-radius: var(--radius-md);
    border: 1px solid var(--stroke-strong);
    background: var(--glass-strong);
    backdrop-filter: blur(18px);
    -webkit-backdrop-filter: blur(18px);
    box-shadow: var(--shadow-lg);
  }
  .opt {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 9px 11px;
    border: none;
    background: transparent;
    color: var(--text);
    font: inherit;
    text-align: left;
    border-radius: 9px;
    cursor: pointer;
    transition: background var(--dur) var(--ease);
  }
  .opt:hover { background: var(--panel); }
  .opt.active { background: var(--accent-soft); color: var(--accent); }
  .opt-hint { font-size: 11px; color: var(--muted); }
</style>
`);

write('src/lib/components/ui/ProgressBar.svelte', `<script lang="ts">
  let { value = 0, indeterminate = false }: { value?: number; indeterminate?: boolean } = $props();
  const pct = $derived(Math.max(0, Math.min(100, value)));
</script>

<div class="track">
  <div class="fill" class:indeterminate style="width: {indeterminate ? 100 : pct}%"></div>
</div>

<style>
  .track {
    width: 100%;
    height: 8px;
    border-radius: 999px;
    background: var(--panel-strong);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    box-shadow: 0 0 16px var(--accent-glow);
    transition: width 180ms var(--ease);
  }
  .fill.indeterminate {
    width: 40% !important;
    animation: indet 1.2s var(--ease) infinite;
  }
  @keyframes indet {
    0% { margin-left: -40%; }
    100% { margin-left: 100%; }
  }
</style>
`);

write('src/lib/components/LoaderSelect.svelte', `<script lang="ts">
  import Dropdown from './ui/Dropdown.svelte';
  import type { Loader } from '../ipc/types';

  let { value = $bindable() }: { value: Loader } = $props();

  const options = [
    { value: 'vanilla', label: 'Vanilla', hint: 'Sin mods' },
    { value: 'fabric', label: 'Fabric', hint: 'Próximamente' },
    { value: 'forge', label: 'Forge', hint: 'Próximamente' },
    { value: 'neoforge', label: 'NeoForge', hint: 'Próximamente' },
  ];
</script>

<Dropdown bind:value {options} />
`);

write('src/lib/components/VersionSelect.svelte', `<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Dropdown from './ui/Dropdown.svelte';
  import type { VersionOption } from '../ipc/types';

  let { value = $bindable() }: { value: string } = $props();

  let versions = $state<VersionOption[]>([]);
  let showSnapshots = $state(false);
  let loading = $state(true);

  const options = $derived(
    versions
      .filter((v) => showSnapshots || v.kind === 'release')
      .map((v) => ({ value: v.id, label: v.id, hint: v.kind === 'release' ? '' : v.kind })),
  );

  async function load() {
    loading = true;
    try {
      versions = await invoke<VersionOption[]>('list_mc_versions');
      if (!value && versions.length) {
        value = versions.find((v) => v.kind === 'release')?.id ?? versions[0].id;
      }
    } catch {
      versions = [];
    } finally {
      loading = false;
    }
  }
  load();
</script>

<div class="vs">
  {#if loading}
    <div class="loading">Cargando versiones…</div>
  {:else}
    <Dropdown bind:value {options} placeholder="Elige versión" />
  {/if}
  <label class="snap">
    <input type="checkbox" bind:checked={showSnapshots} />
    <span>Mostrar snapshots</span>
  </label>
</div>

<style>
  .vs { display: flex; flex-direction: column; gap: 8px; }
  .loading {
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--stroke);
    background: var(--panel);
    color: var(--muted);
    font-size: 13px;
  }
  .snap { display: flex; align-items: center; gap: 7px; font-size: 12px; color: var(--muted); cursor: pointer; }
  .snap input { accent-color: var(--accent); }
</style>
`);

write('src/lib/stores/install.svelte.ts', `import { invoke, Channel } from '@tauri-apps/api/core';
import type { InstallEvent } from '../ipc/types';
import { instances } from './instances.svelte';

// Estado de una instalacion en curso. La UI se suscribe a este store.
class InstallStore {
  activeId = $state<string | null>(null);
  phase = $state<string>('');
  filesDone = $state(0);
  totalFiles = $state(0);
  bytesDone = $state(0);
  totalBytes = $state(0);
  currentFile = $state('');
  speedBps = $state(0);
  error = $state<string | null>(null);

  percent = $derived(this.totalBytes > 0 ? (this.bytesDone / this.totalBytes) * 100 : 0);

  isInstalling(id: string) {
    return this.activeId === id;
  }

  reset() {
    this.phase = '';
    this.filesDone = 0;
    this.totalFiles = 0;
    this.bytesDone = 0;
    this.totalBytes = 0;
    this.currentFile = '';
    this.speedBps = 0;
    this.error = null;
  }

  async start(id: string) {
    this.reset();
    this.activeId = id;

    const channel = new Channel<InstallEvent>();
    channel.onmessage = (e) => {
      switch (e.kind) {
        case 'started':
          this.totalFiles = e.total_files;
          this.totalBytes = e.total_bytes;
          break;
        case 'phase':
          this.phase = e.phase;
          break;
        case 'progress':
          this.filesDone = e.files_done;
          this.totalFiles = e.total_files;
          this.bytesDone = e.bytes_done;
          this.totalBytes = e.total_bytes;
          this.currentFile = e.current_file;
          this.speedBps = e.speed_bps;
          break;
        case 'failed':
          this.error = e.message;
          break;
      }
    };

    try {
      await invoke('install_instance', { id, onEvent: channel });
    } catch (err: any) {
      this.error = String(err?.message ?? err);
    } finally {
      this.activeId = null;
      await instances.load();
    }
  }

  async cancel(id: string) {
    await invoke('cancel_install', { id });
  }
}

export const install = new InstallStore();
`);

write('src/lib/components/InstallProgress.svelte', `<script lang="ts">
  import ProgressBar from './ui/ProgressBar.svelte';
  import Button from './ui/Button.svelte';
  import { install } from '../stores/install.svelte';
  import { PHASE_LABEL } from '../ipc/types';
  import { fade } from 'svelte/transition';

  let { id }: { id: string } = $props();

  function fmtBytes(n: number): string {
    if (n < 1024) return n + ' B';
    if (n < 1024 * 1024) return (n / 1024).toFixed(0) + ' KB';
    if (n < 1024 * 1024 * 1024) return (n / 1024 / 1024).toFixed(1) + ' MB';
    return (n / 1024 / 1024 / 1024).toFixed(2) + ' GB';
  }
  const speed = $derived(fmtBytes(install.speedBps) + '/s');
  const eta = $derived(() => {
    if (install.speedBps <= 0) return '—';
    const remaining = install.totalBytes - install.bytesDone;
    const secs = Math.max(0, Math.round(remaining / install.speedBps));
    if (secs < 60) return secs + 's';
    return Math.floor(secs / 60) + 'm ' + (secs % 60) + 's';
  });
</script>

<div class="ip" in:fade={{ duration: 200 }}>
  <div class="head">
    <span class="phase">{PHASE_LABEL[install.phase] ?? 'Instalando'}</span>
    <span class="pct">{install.percent.toFixed(0)}%</span>
  </div>

  <ProgressBar value={install.percent} indeterminate={install.totalBytes === 0} />

  <div class="detail">
    <span class="file" title={install.currentFile}>{install.currentFile || '…'}</span>
    <span class="nums">{install.filesDone}/{install.totalFiles}</span>
  </div>

  <div class="foot">
    <div class="meta">
      <span>{fmtBytes(install.bytesDone)} / {fmtBytes(install.totalBytes)}</span>
      <span class="dot">•</span>
      <span>{speed}</span>
      <span class="dot">•</span>
      <span>ETA {eta()}</span>
    </div>
    <Button variant="ghost" onclick={() => install.cancel(id)}>Cancelar</Button>
  </div>
</div>

<style>
  .ip { display: flex; flex-direction: column; gap: 12px; }
  .head { display: flex; align-items: center; justify-content: space-between; }
  .phase { font-weight: 700; font-size: 14px; }
  .pct { font-weight: 800; font-size: 15px; color: var(--accent); }
  .detail { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .file {
    font-size: 12px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: ui-monospace, monospace;
  }
  .nums { font-size: 12px; color: var(--muted); flex: 0 0 auto; }
  .foot { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .meta { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--muted); }
  .dot { opacity: 0.5; }
</style>
`);

write('src/lib/components/CreateInstanceModal.svelte', `<script lang="ts">
  import Modal from './ui/Modal.svelte';
  import Button from './ui/Button.svelte';
  import VersionSelect from './VersionSelect.svelte';
  import LoaderSelect from './LoaderSelect.svelte';
  import { instances } from '../stores/instances.svelte';
  import type { Instance, Loader } from '../ipc/types';

  let { open = false, edit = null, onclose }: {
    open?: boolean;
    edit?: Instance | null;
    onclose?: () => void;
  } = $props();

  let name = $state('');
  let mcVersion = $state('');
  let loader = $state<Loader>('vanilla');
  let ram = $state(4096);
  let saving = $state(false);
  let error = $state<string | null>(null);

  const isEdit = $derived(!!edit);

  $effect(() => {
    if (open) {
      error = null;
      if (edit) {
        name = edit.name;
        mcVersion = edit.mc_version;
        loader = edit.loader;
        ram = edit.ram_mb;
      } else {
        name = '';
        mcVersion = '';
        loader = 'vanilla';
        ram = 4096;
      }
    }
  });

  async function submit() {
    if (!name.trim()) { error = 'Ponle un nombre a la instancia.'; return; }
    if (!mcVersion) { error = 'Elige una versión de Minecraft.'; return; }
    if (loader !== 'vanilla') {
      error = 'Por ahora solo Vanilla está disponible. El resto llega pronto.';
      return;
    }
    saving = true;
    error = null;
    try {
      if (isEdit && edit) {
        await instances.update(edit.id, { name: name.trim(), mc_version: mcVersion, loader, ram_mb: ram });
      } else {
        await instances.create({ name: name.trim(), mc_version: mcVersion, loader, ram_mb: ram });
      }
      onclose?.();
    } catch (e: any) {
      error = String(e?.message ?? e);
    } finally {
      saving = false;
    }
  }
</script>

<Modal {open} title={isEdit ? 'Editar instancia' : 'Nueva instancia'} onclose={() => onclose?.()}>
  <div class="form">
    <label>
      <span>Nombre</span>
      <input type="text" bind:value={name} placeholder="Mi mundo" />
    </label>

    <div class="row">
      <div class="field">
        <span class="lbl">Versión de Minecraft</span>
        <VersionSelect bind:value={mcVersion} />
      </div>
      <div class="field">
        <span class="lbl">Loader</span>
        <LoaderSelect bind:value={loader} />
      </div>
    </div>

    <label>
      <span>Memoria asignada: <b>{(ram / 1024).toFixed(1)} GB</b></span>
      <input type="range" min="1024" max="16384" step="512" bind:value={ram} />
    </label>

    {#if error}<p class="err">{error}</p>{/if}
  </div>

  {#snippet footer()}
    <Button variant="ghost" onclick={() => onclose?.()}>Cancelar</Button>
    <Button variant="primary" disabled={saving} onclick={submit}>
      {saving ? 'Guardando…' : isEdit ? 'Guardar cambios' : 'Crear instancia'}
    </Button>
  {/snippet}
</Modal>

<style>
  .form { display: flex; flex-direction: column; gap: 16px; }
  .row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; align-items: start; }
  .field { display: flex; flex-direction: column; gap: 7px; }
  label { display: flex; flex-direction: column; gap: 7px; }
  label > span, .lbl { font-size: 13px; font-weight: 600; color: var(--text); }
  input[type='text'] {
    font: inherit;
    color: var(--text);
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--stroke);
    background: var(--panel);
    transition: border-color var(--dur) var(--ease), background var(--dur) var(--ease);
  }
  input[type='text']:focus { outline: none; border-color: var(--accent); background: var(--panel-strong); }
  input[type='range'] { width: 100%; accent-color: var(--accent); }
  .err { color: #ff7a8a; font-size: 13px; }
</style>
`);

write('src/lib/components/InstanceCard.svelte', `<script lang="ts">
  import Icon from './ui/Icon.svelte';
  import Badge from './ui/Badge.svelte';
  import ProgressBar from './ui/ProgressBar.svelte';
  import { LOADER_LABEL, STATUS_LABEL, PHASE_LABEL } from '../ipc/types';
  import type { InstanceSummary, InstallStatus } from '../ipc/types';
  import { install } from '../stores/install.svelte';

  let { instance, onplay, onedit, onduplicate, ondelete, onfavorite, oninstall }: {
    instance: InstanceSummary;
    onplay: () => void;
    onedit: () => void;
    onduplicate: () => void;
    ondelete: () => void;
    onfavorite: () => void;
    oninstall: () => void;
  } = $props();

  const statusVariant: Record<InstallStatus, 'neutral' | 'accent' | 'ok' | 'warn'> = {
    created: 'warn',
    installing: 'accent',
    cancelled: 'warn',
    ready: 'ok',
    error: 'warn',
    corrupt: 'warn',
  };

  const installing = $derived(install.isInstalling(instance.id));
  const isReady = $derived(instance.status === 'ready');
</script>

<div class="card">
  <button class="fav" class:on={instance.favorite} onclick={onfavorite}
    title={instance.favorite ? 'Quitar de favoritas' : 'Marcar como favorita'}>
    <Icon name="star" size={16} />
  </button>

  <div class="thumb">{instance.name.charAt(0).toUpperCase()}</div>

  <div class="info">
    <b class="name" title={instance.name}>{instance.name}</b>
    <div class="tags">
      <span class="tag">{instance.mc_version}</span>
      <span class="tag">{LOADER_LABEL[instance.loader]}</span>
    </div>
    <div class="status">
      <Badge variant={statusVariant[instance.status]}>{STATUS_LABEL[instance.status]}</Badge>
      <span class="mods">{instance.mod_count} mods</span>
    </div>
  </div>

  {#if installing}
    <div class="installing">
      <div class="ins-head">
        <span>{PHASE_LABEL[install.phase] ?? 'Instalando'}</span>
        <span class="ins-pct">{install.percent.toFixed(0)}%</span>
      </div>
      <ProgressBar value={install.percent} indeterminate={install.totalBytes === 0} />
      <button class="cancel" onclick={() => install.cancel(instance.id)}>Cancelar</button>
    </div>
  {:else}
    <div class="actions">
      {#if isReady}
        <button class="play" onclick={onplay}><Icon name="play" size={15} /> Jugar</button>
      {:else}
        <button class="play install" onclick={oninstall}>
          <Icon name="download" size={15} /> Instalar
        </button>
      {/if}
      <div class="icons">
        <button title="Editar" onclick={onedit}><Icon name="pencil" size={15} /></button>
        <button title="Duplicar" onclick={onduplicate}><Icon name="copy" size={15} /></button>
        <button class="danger" title="Eliminar" onclick={ondelete}><Icon name="trash" size={15} /></button>
      </div>
    </div>
  {/if}
</div>

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--stroke);
    background: var(--glass);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    box-shadow: var(--shadow-soft);
    transition: transform var(--dur) var(--ease), border-color var(--dur) var(--ease);
  }
  .card:hover { transform: translateY(-3px); border-color: var(--stroke-strong); }

  .fav {
    position: absolute; top: 14px; right: 14px;
    display: grid; place-items: center; width: 30px; height: 30px;
    border: none; border-radius: 9px; background: var(--panel); color: var(--muted); cursor: pointer;
    transition: color var(--dur) var(--ease), background var(--dur) var(--ease);
  }
  .fav:hover { background: var(--panel-strong); color: var(--text); }
  .fav.on { color: var(--warn); }

  .thumb {
    display: grid; place-items: center; width: 54px; height: 54px;
    border-radius: 14px; font-size: 24px; font-weight: 800; color: #fff;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: 0 8px 24px var(--accent-glow);
  }

  .info { display: flex; flex-direction: column; gap: 8px; min-width: 0; }
  .name { font-size: 15px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tags { display: flex; flex-wrap: wrap; gap: 6px; }
  .tag { font-size: 11.5px; padding: 3px 9px; border-radius: 999px; color: var(--muted); background: var(--panel); border: 1px solid var(--stroke); }
  .status { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .mods { font-size: 12px; color: var(--muted); }

  .actions { display: flex; align-items: center; gap: 8px; margin-top: 2px; }
  .play {
    flex: 1; display: inline-flex; align-items: center; justify-content: center; gap: 7px;
    padding: 10px; border: none; border-radius: var(--radius-md);
    font: inherit; font-weight: 700; font-size: 13px; color: #fff; cursor: pointer;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: 0 8px 22px var(--accent-glow);
    transition: transform var(--dur) var(--ease), box-shadow var(--dur) var(--ease);
  }
  .play:hover { transform: translateY(-2px); box-shadow: 0 12px 30px var(--accent-glow); }
  .play:active { transform: translateY(0) scale(0.99); }
  .play.install { background: var(--panel-strong); color: var(--text); box-shadow: none; border: 1px solid var(--stroke-strong); }
  .play.install:hover { background: var(--panel); box-shadow: none; }

  .icons { display: flex; gap: 4px; }
  .icons button {
    display: grid; place-items: center; width: 36px; height: 36px;
    border: 1px solid var(--stroke); border-radius: var(--radius-md);
    background: var(--panel); color: var(--muted); cursor: pointer;
    transition: color var(--dur) var(--ease), background var(--dur) var(--ease), border-color var(--dur) var(--ease);
  }
  .icons button:hover { background: var(--panel-strong); color: var(--text); border-color: var(--stroke-strong); }
  .icons button.danger:hover { background: rgba(229, 72, 77, 0.16); color: #ff7a8a; border-color: transparent; }

  .installing { display: flex; flex-direction: column; gap: 8px; }
  .ins-head { display: flex; align-items: center; justify-content: space-between; font-size: 12.5px; }
  .ins-pct { font-weight: 800; color: var(--accent); }
  .cancel {
    align-self: flex-end; margin-top: 2px; padding: 5px 12px;
    border: 1px solid var(--stroke); border-radius: 8px;
    background: transparent; color: var(--muted); font: inherit; font-size: 12px; cursor: pointer;
    transition: color var(--dur) var(--ease), border-color var(--dur) var(--ease);
  }
  .cancel:hover { color: #ff7a8a; border-color: rgba(229, 72, 77, 0.4); }
</style>
`);

write('src/lib/stores/instances.svelte.ts', `import { invoke } from '@tauri-apps/api/core';
import type { Instance, InstanceSummary, CreateInput } from '../ipc/types';

// Rust es la fuente de verdad. Este store cachea el indice en memoria.
class InstancesStore {
  items = $state<InstanceSummary[]>([]);
  selectedId = $state<string | null>(null);
  loading = $state(false);
  loaded = $state(false);

  selected = $derived(this.items.find((i) => i.id === this.selectedId) ?? null);

  async load() {
    this.loading = true;
    try {
      this.items = await invoke<InstanceSummary[]>('list_instances');
      if ((!this.selectedId || !this.items.some((i) => i.id === this.selectedId)) && this.items.length) {
        const fav = this.items.find((i) => i.favorite);
        const recent = [...this.items].sort((a, b) => (b.last_played ?? 0) - (a.last_played ?? 0))[0];
        this.selectedId = (fav ?? recent ?? this.items[0]).id;
      }
    } finally {
      this.loading = false;
      this.loaded = true;
    }
  }

  select(id: string) { this.selectedId = id; }
  get(id: string): Promise<Instance> { return invoke<Instance>('get_instance', { id }); }

  async create(input: CreateInput): Promise<Instance> {
    const created = await invoke<Instance>('create_instance', { input });
    await this.load();
    this.selectedId = created.id;
    return created;
  }
  async update(id: string, patch: Record<string, unknown>): Promise<Instance> {
    const updated = await invoke<Instance>('update_instance', { id, patch });
    await this.load();
    return updated;
  }
  async remove(id: string): Promise<void> {
    await invoke('delete_instance', { id });
    if (this.selectedId === id) this.selectedId = null;
    await this.load();
  }
  async duplicate(id: string): Promise<Instance> {
    const copy = await invoke<Instance>('duplicate_instance', { id });
    await this.load();
    return copy;
  }
  toggleFavorite(id: string, value: boolean) { return this.update(id, { favorite: value }); }
}

export const instances = new InstancesStore();
`);

write('src/lib/components/ui/Icon.svelte', `<script lang="ts">
  let { name, size = 20 }: { name: string; size?: number } = $props();

  const paths: Record<string, string> = {
    home: 'M3 10.5 12 3l9 7.5M5 9.5V21h14V9.5',
    grid: 'M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z',
    settings:
      'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z',
    sun: 'M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42',
    moon: 'M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z',
    minus: 'M5 12h14',
    square: 'M5 5h14v14H5z',
    close: 'M18 6 6 18M6 6l12 12',
    play: 'M7 4.5v15l12-7.5z',
    clock: 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zM12 6.5V12l4 2',
    package: 'M21 16V8l-9-5-9 5v8l9 5 9-5zM3 8l9 5 9-5M12 13v8',
    download: 'M12 3v12M7 10l5 5 5-5M5 21h14',
    user: 'M20 21a8 8 0 1 0-16 0M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',
    plus: 'M12 5v14M5 12h14',
    folder: 'M3 7a2 2 0 0 1 2-2h4l2 3h8a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z',
    chevron: 'M9 6l6 6-6 6',
    layers: 'M12 3 2 8l10 5 10-5-10-5zM2 13l10 5 10-5M2 18l10 5 10-5',
    star: 'M12 3l2.9 5.9 6.5.9-4.7 4.6 1.1 6.5L12 18.3 6.2 21.4l1.1-6.5L2.6 9.8l6.5-.9L12 3z',
    copy: 'M9 9h10v10H9zM5 15H4V5a1 1 0 0 1 1-1h10v1',
    pencil: 'M12 20h9M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z',
    trash: 'M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14',
    import: 'M12 3v11M8 10l4 4 4-4M4 15v4a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-4',
  };
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none"
  stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <path d={paths[name] ?? ''} />
</svg>
`);

write('src/lib/views/HomeView.svelte', `<script lang="ts">
  import { onMount } from 'svelte';
  import Panel from '../components/ui/Panel.svelte';
  import Badge from '../components/ui/Badge.svelte';
  import Icon from '../components/ui/Icon.svelte';
  import InstallProgress from '../components/InstallProgress.svelte';
  import { router } from '../stores/router.svelte';
  import { instances } from '../stores/instances.svelte';
  import { install } from '../stores/install.svelte';
  import { LOADER_LABEL, STATUS_LABEL } from '../ipc/types';
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  onMount(() => { if (!instances.loaded) instances.load(); });

  const sel = $derived(instances.selected);
  const total = $derived(instances.items.length);
  const totalMods = $derived(instances.items.reduce((a, i) => a + i.mod_count, 0));
  const totalHours = $derived(Math.floor(instances.items.reduce((a, i) => a + i.playtime_secs, 0) / 3600));
  const totalGb = $derived(
    (instances.items.reduce((a, i) => a + (i.total_size_bytes ?? 0), 0) / 1024 / 1024 / 1024).toFixed(1),
  );
  const selInstalling = $derived(!!sel && install.isInstalling(sel.id));

  const quick = [
    { icon: 'plus', label: 'Nueva instancia', primary: true, go: () => router.navigate('instances') },
    { icon: 'grid', label: 'Mis instancias', primary: false, go: () => router.navigate('instances') },
    { icon: 'package', label: 'Mods', primary: false, go: () => router.navigate('instances') },
    { icon: 'settings', label: 'Ajustes', primary: false, go: () => router.navigate('settings') },
  ];

  function lastPlayedLabel(ts: number | null): string {
    if (!ts) return 'Nunca';
    return new Date(ts * 1000).toLocaleDateString();
  }

  function primaryAction() {
    if (!sel) { router.navigate('instances'); return; }
    if (sel.status === 'ready') {
      // El lanzamiento real de la JVM llega en la Fase 4.
      return;
    }
    install.start(sel.id);
  }

  const ctaLabel = $derived(!sel ? 'CREAR INSTANCIA' : sel.status === 'ready' ? 'JUGAR' : 'INSTALAR');
  const ctaSub = $derived(
    !sel ? 'Configura tu primer perfil'
      : sel.status === 'ready' ? sel.mc_version
      : sel.status === 'error' ? 'Reintentar instalación'
      : 'Descargar ' + sel.mc_version,
  );
</script>

<div class="home">
  <section class="featured" in:fly={{ y: 18, duration: 380, easing: cubicOut }}>
    <div class="featured-glow"></div>
    <div class="featured-info">
      {#if sel}
        <span class="eyebrow">{sel.favorite ? 'Tu favorita' : 'Continúa jugando'}</span>
        <h1>{sel.name}</h1>
        <div class="meta">
          <span class="chip"><Icon name="layers" size={14} /> {sel.mc_version}</span>
          <span class="chip"><Icon name="package" size={14} /> {LOADER_LABEL[sel.loader]}</span>
          <span class="chip"><Icon name="package" size={14} /> {sel.mod_count} mods</span>
          <span class="chip"><Icon name="clock" size={14} /> {lastPlayedLabel(sel.last_played)}</span>
        </div>
      {:else}
        <span class="eyebrow">Empieza aquí</span>
        <h1>Tu primera instancia te espera</h1>
        <p class="featured-sub">Crea un perfil, elige tu versión y despega en segundos.</p>
      {/if}
    </div>

    {#if !selInstalling}
      <div class="featured-cta">
        {#if sel}<Badge variant={sel.status === 'ready' ? 'ok' : 'warn'}>{STATUS_LABEL[sel.status]}</Badge>{/if}
        <button class="play" onclick={primaryAction}>
          <Icon name={sel && sel.status === 'ready' ? 'play' : sel ? 'download' : 'plus'} size={28} />
          <span class="play-text">
            <b>{ctaLabel}</b>
            <small>{ctaSub}</small>
          </span>
        </button>
      </div>
    {/if}
  </section>

  {#if selInstalling && sel}
    <section in:fly={{ y: 14, duration: 320, easing: cubicOut }}>
      <Panel title="Instalando {sel.name}" description="No cierres Aether hasta terminar.">
        <InstallProgress id={sel.id} />
      </Panel>
    </section>
  {/if}

  <section class="quick" in:fly={{ y: 18, duration: 380, delay: 80, easing: cubicOut }}>
    {#each quick as q, i (q.label)}
      <button class="q-card" class:primary={q.primary} onclick={q.go} style="--d:{i * 45}ms">
        <span class="q-icon"><Icon name={q.icon} size={20} /></span>
        <span class="q-label">{q.label}</span>
        <span class="q-arrow"><Icon name="chevron" size={16} /></span>
      </button>
    {/each}
  </section>

  <div class="grid">
    <div in:fly={{ y: 18, duration: 380, delay: 150, easing: cubicOut }}>
      <Panel title="Actividad reciente" description="Tu historial aparecerá aquí cuando juegues.">
        <div class="empty-row">
          <span class="empty-ic"><Icon name="clock" size={22} /></span>
          <div>
            <b>Tu historial te espera</b>
            <p>Cuando juegues, verás aquí tus últimas sesiones.</p>
          </div>
        </div>
      </Panel>
    </div>

    <div in:fly={{ y: 18, duration: 380, delay: 210, easing: cubicOut }}>
      <Panel title="Novedades" description="Noticias y actualizaciones de Aether.">
        {#snippet action()}<Badge variant="accent">Próximamente</Badge>{/snippet}
        <article class="news-item">
          <span class="news-dot"></span>
          <div>
            <b>Bienvenido a Aether Launcher</b>
            <p>Estamos construyendo el launcher más rápido y elegante. Pronto: noticias en vivo.</p>
          </div>
        </article>
      </Panel>
    </div>

    <div class="span-2" in:fly={{ y: 18, duration: 380, delay: 270, easing: cubicOut }}>
      <Panel title="Estadísticas" description="Tu resumen crecerá a medida que juegues.">
        <div class="stats">
          <div class="stat"><span class="stat-ic"><Icon name="grid" size={18} /></span><b>{total}</b><span>Instancias</span></div>
          <div class="stat"><span class="stat-ic"><Icon name="package" size={18} /></span><b>{totalMods}</b><span>Mods</span></div>
          <div class="stat"><span class="stat-ic"><Icon name="clock" size={18} /></span><b>{totalHours} h</b><span>Jugadas</span></div>
          <div class="stat"><span class="stat-ic"><Icon name="download" size={18} /></span><b>{totalGb} GB</b><span>Descargado</span></div>
        </div>
      </Panel>
    </div>
  </div>
</div>

<style>
  .home { display: flex; flex-direction: column; gap: 22px; max-width: 1140px; margin: 0 auto; }

  .featured {
    position: relative; display: flex; align-items: center; justify-content: space-between;
    gap: 24px; padding: 38px 40px; border-radius: var(--radius-xl);
    border: 1px solid var(--stroke);
    background: linear-gradient(120deg, var(--accent-soft), transparent 60%), var(--glass);
    backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);
    box-shadow: var(--shadow-soft); overflow: hidden;
  }
  .featured-glow {
    position: absolute; top: -40%; right: -10%; width: 380px; height: 380px; border-radius: 50%;
    background: radial-gradient(circle, var(--accent-glow), transparent 70%); filter: blur(20px); pointer-events: none;
  }
  .featured-info { position: relative; z-index: 1; min-width: 0; }
  .eyebrow { display: inline-block; font-size: 11px; letter-spacing: 2px; text-transform: uppercase; color: var(--accent); margin-bottom: 10px; }
  .featured h1 { font-size: 30px; font-weight: 800; letter-spacing: -0.5px; }
  .featured-sub { color: var(--muted); font-size: 15px; margin-top: 12px; max-width: 420px; }
  .meta { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 16px; }
  .chip { display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 999px; font-size: 12.5px; color: var(--muted); background: var(--panel); border: 1px solid var(--stroke); }

  .featured-cta { position: relative; z-index: 1; display: flex; flex-direction: column; align-items: flex-end; gap: 12px; flex: 0 0 auto; }
  .play {
    display: inline-flex; align-items: center; gap: 15px; padding: 22px 38px;
    border: none; border-radius: var(--radius-lg); color: #fff; cursor: pointer;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: 0 16px 48px var(--accent-glow);
    transition: transform var(--dur) var(--ease), box-shadow var(--dur) var(--ease);
  }
  .play:hover { transform: translateY(-3px) scale(1.02); box-shadow: 0 22px 64px var(--accent-glow); }
  .play:active { transform: translateY(-1px) scale(0.99); }
  .play-text { display: flex; flex-direction: column; align-items: flex-start; line-height: 1.1; }
  .play-text b { font-size: 24px; font-weight: 800; letter-spacing: 1.5px; }
  .play-text small { font-size: 12px; opacity: 0.85; }

  .quick { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; }
  .q-card {
    display: flex; align-items: center; gap: 12px; padding: 16px 18px;
    border: 1px solid var(--stroke); border-radius: var(--radius-lg); background: var(--glass);
    backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); color: var(--text); cursor: pointer; text-align: left;
    animation: rise 0.42s var(--ease) both; animation-delay: var(--d);
    transition: transform var(--dur) var(--ease), border-color var(--dur) var(--ease), background var(--dur) var(--ease);
  }
  .q-card:hover { transform: translateY(-3px); border-color: var(--stroke-strong); background: var(--glass-strong); }
  .q-card:active { transform: translateY(-1px) scale(0.99); }
  .q-card.primary { border-color: transparent; background: var(--accent-soft); }
  .q-card.primary .q-label { color: var(--accent); }
  .q-icon { display: grid; place-items: center; width: 40px; height: 40px; border-radius: 12px; color: var(--accent); background: var(--accent-soft); flex: 0 0 auto; }
  .q-card.primary .q-icon { color: #fff; background: linear-gradient(135deg, var(--accent), var(--accent-2)); }
  .q-label { flex: 1; font-weight: 600; font-size: 13.5px; }
  .q-arrow { color: var(--muted); transition: transform var(--dur) var(--ease); }
  .q-card:hover .q-arrow { transform: translateX(3px); color: var(--text); }

  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 18px; }
  .grid > div { min-width: 0; }
  .span-2 { grid-column: 1 / -1; }

  .empty-row { display: flex; align-items: center; gap: 14px; padding: 8px 0; }
  .empty-ic { display: grid; place-items: center; width: 46px; height: 46px; border-radius: 13px; color: var(--accent); background: var(--accent-soft); flex: 0 0 auto; }
  .empty-row b { font-size: 14px; }
  .empty-row p { color: var(--muted); font-size: 13px; margin-top: 2px; }

  .news-item { display: flex; gap: 12px; }
  .news-dot { width: 9px; height: 9px; margin-top: 6px; border-radius: 50%; background: linear-gradient(var(--accent), var(--accent-2)); flex: 0 0 auto; box-shadow: 0 0 12px var(--accent-glow); }
  .news-item b { font-size: 14px; }
  .news-item p { color: var(--muted); font-size: 13px; margin-top: 3px; }

  .stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; }
  .stat { display: flex; flex-direction: column; gap: 4px; padding: 16px; border-radius: var(--radius-md); background: var(--panel); border: 1px solid var(--stroke); }
  .stat-ic { color: var(--accent); margin-bottom: 4px; }
  .stat b { font-size: 22px; font-weight: 800; }
  .stat span { color: var(--muted); font-size: 12px; }

  @keyframes rise { from { opacity: 0; transform: translateY(12px); } to { opacity: 1; transform: translateY(0); } }

  @media (max-width: 880px) {
    .quick { grid-template-columns: repeat(2, 1fr); }
    .grid { grid-template-columns: 1fr; }
    .stats { grid-template-columns: repeat(2, 1fr); }
    .featured { flex-direction: column; align-items: flex-start; }
    .featured-cta { align-items: flex-start; }
  }
</style>
`);

write('src/lib/views/InstancesView.svelte', `<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '../components/ui/Button.svelte';
  import Modal from '../components/ui/Modal.svelte';
  import InstanceCard from '../components/InstanceCard.svelte';
  import CreateInstanceModal from '../components/CreateInstanceModal.svelte';
  import { instances } from '../stores/instances.svelte';
  import { install } from '../stores/install.svelte';
  import { router } from '../stores/router.svelte';
  import type { Instance, InstanceSummary } from '../ipc/types';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  let showForm = $state(false);
  let editing = $state<Instance | null>(null);
  let toDelete = $state<InstanceSummary | null>(null);
  let busy = $state(false);

  onMount(() => { if (!instances.loaded) instances.load(); });

  function openCreate() { editing = null; showForm = true; }
  async function openEdit(s: InstanceSummary) { editing = await instances.get(s.id); showForm = true; }
  function playOne(s: InstanceSummary) { instances.select(s.id); router.navigate('home'); }
  function installOne(s: InstanceSummary) { instances.select(s.id); install.start(s.id); }

  async function confirmDelete() {
    if (!toDelete) return;
    busy = true;
    try { await instances.remove(toDelete.id); toDelete = null; }
    finally { busy = false; }
  }
</script>

<div class="page">
  <header class="page-head" in:fly={{ y: 14, duration: 320, easing: cubicOut }}>
    <div>
      <h2>Instancias</h2>
      <p>Gestiona tus perfiles de Minecraft.</p>
    </div>
    <Button variant="glass" onclick={openCreate}>+ Nueva instancia</Button>
  </header>

  {#if instances.items.length === 0}
    <div class="empty" in:fade={{ duration: 260 }}>
      <div class="orb"></div>
      <h3>Tu primera instancia te espera</h3>
      <p>Crea un perfil, elige versión y loader, y prepárate para jugar.</p>
      <div class="empty-cta"><Button variant="primary" onclick={openCreate}>Crear instancia</Button></div>
    </div>
  {:else}
    <div class="grid">
      {#each instances.items as inst, i (inst.id)}
        <div in:fly={{ y: 16, duration: 320, delay: i * 45, easing: cubicOut }}>
          <InstanceCard
            instance={inst}
            onplay={() => playOne(inst)}
            oninstall={() => installOne(inst)}
            onedit={() => openEdit(inst)}
            onduplicate={() => instances.duplicate(inst.id)}
            ondelete={() => (toDelete = inst)}
            onfavorite={() => instances.toggleFavorite(inst.id, !inst.favorite)} />
        </div>
      {/each}
    </div>
  {/if}
</div>

<CreateInstanceModal open={showForm} edit={editing} onclose={() => (showForm = false)} />

<Modal open={!!toDelete} title="Eliminar instancia" onclose={() => (toDelete = null)}>
  <p class="del-note">
    ¿Seguro que quieres eliminar <b>{toDelete?.name}</b>? Se borrarán todos sus
    archivos de forma permanente. Esta acción no se puede deshacer.
  </p>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (toDelete = null)}>Cancelar</Button>
    <Button variant="primary" disabled={busy} onclick={confirmDelete}>{busy ? 'Eliminando…' : 'Eliminar'}</Button>
  {/snippet}
</Modal>

<style>
  .page { display: flex; flex-direction: column; height: 100%; max-width: 1100px; margin: 0 auto; }
  .page-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; margin-bottom: 28px; }
  .page-head h2 { font-size: 26px; font-weight: 700; }
  .page-head p { color: var(--muted); margin-top: 4px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 18px; align-content: start; }
  .empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; gap: 6px; padding-bottom: 40px; }
  .orb { width: 92px; height: 92px; border-radius: 50%; margin-bottom: 20px; background: radial-gradient(circle at 30% 30%, var(--accent), var(--accent-2)); box-shadow: 0 0 60px var(--accent-glow); animation: float 4s ease-in-out infinite; }
  @keyframes float { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-10px); } }
  .empty h3 { font-size: 19px; font-weight: 700; }
  .empty p { color: var(--muted); max-width: 360px; }
  .empty-cta { margin-top: 18px; }
  .del-note { color: var(--muted); font-size: 14px; line-height: 1.6; }
  .del-note b { color: var(--text); }
</style>
`);

console.log('\nFase 3 aplicada. IMPORTANTE: se modifico Rust, reinicia npx tauri dev.\n');