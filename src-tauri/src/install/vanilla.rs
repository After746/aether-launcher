//! Instalador Vanilla completo. Orden por capas:
//! manifest -> version JSON -> libraries -> client jar -> assets -> natives.

use super::{InstallEvent, Installer};
use crate::download::{run_batch, DownloadTask, Progress};
use crate::error::Result;
use crate::instance::store;
use crate::instance::Instance;
use crate::minecraft::manifest::VersionManifest;
use crate::minecraft::version::{AssetIndex, VersionDetail};
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
