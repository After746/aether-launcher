//! Instalacion de un runtime de Java administrado. Reutiliza el pool de
//! descargas existente (download::run_batch, verificacion SHA1, skip-if-valid)
//! y solo agrega lo especifico de un runtime: symlinks y bit de ejecucion.
//!
//! No duplica el instalador Vanilla: comparte el mismo motor de descargas.

use crate::download::{run_batch, DownloadTask, Progress};
use crate::error::{AetherError, Result};
use crate::runtime::manifest::{self, FileEntry};
use crate::runtime::paths;
use crate::runtime::{detect, ManagedRuntime, RuntimeEvent};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

/// Marca un archivo como ejecutable (0o755) en Unix; no-op en Windows.
#[cfg(unix)]
fn mark_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}
#[cfg(not(unix))]
fn mark_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Crea un symlink relativo. En Windows, donde los runtimes rara vez usan
/// links, se degrada a copiar el archivo destino si existe.
fn make_link(link: &Path, target: &str) -> Result<()> {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }
    #[cfg(unix)]
    {
        if link.exists() {
            let _ = std::fs::remove_file(link);
        }
        std::os::unix::fs::symlink(target, link)?;
    }
    #[cfg(windows)]
    {
        let resolved = link.parent().map(|p| p.join(target)).unwrap_or_default();
        if resolved.exists() {
            std::fs::copy(&resolved, link)?;
        }
    }
    Ok(())
}

/// Descarga, verifica y extrae el runtime de Java para `major` dentro de
/// `<data_dir>/runtimes/java-<major>/`. Devuelve el runtime administrado listo.
pub async fn download_runtime(
    client: &reqwest::Client,
    major: u32,
    on_event: &Channel<RuntimeEvent>,
    cancel: CancellationToken,
) -> Result<ManagedRuntime> {
    // --- Fase A: resolver plataforma y componente ---
    let _ = on_event.send(RuntimeEvent::Phase { phase: "index".into() });
    let platform_key = manifest::mojang_platform_key()?;
    let index = manifest::fetch_index(client).await?;
    let release = manifest::select_release(&index, platform_key, major)?;

    let _ = on_event.send(RuntimeEvent::Phase { phase: "manifest".into() });
    let man = manifest::fetch_manifest(client, &release.manifest.url).await?;

    // --- Fase B: planificar archivos, directorios y links ---
    let home = paths::runtime_home_for(major)?;
    std::fs::create_dir_all(&home)?;

    let mut tasks: Vec<DownloadTask> = Vec::new();
    let mut exec_files: Vec<PathBuf> = Vec::new();
    let mut links: Vec<(PathBuf, String)> = Vec::new();

    for (rel_path, entry) in &man.files {
        let dest = home.join(rel_path);
        match entry {
            FileEntry::Directory => {
                std::fs::create_dir_all(&dest)?;
            }
            FileEntry::Link { target } => {
                links.push((dest, target.clone()));
            }
            FileEntry::File { executable, downloads } => {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                tasks.push(DownloadTask {
                    url: downloads.raw.url.clone(),
                    dest: dest.clone(),
                    sha1: Some(downloads.raw.sha1.clone()),
                    size: downloads.raw.size,
                });
                if *executable {
                    exec_files.push(dest);
                }
            }
        }
    }

    // --- Fase C: descarga (motor existente) con progreso ---
    let progress = Arc::new(Progress::default());
    let total_files = tasks.len() as u64;
    let total_bytes: u64 = tasks.iter().map(|t| t.size).sum();
    progress.total_files.store(total_files, Ordering::Relaxed);
    progress.total_bytes.store(total_bytes, Ordering::Relaxed);
    let _ = on_event.send(RuntimeEvent::Started {
        major,
        version: release.version.name.clone(),
        total_files,
        total_bytes,
    });
    let _ = on_event.send(RuntimeEvent::Phase { phase: "download".into() });

    let current = Arc::new(tokio::sync::Mutex::new(String::new()));
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
                let _ = on_event.send(RuntimeEvent::Progress {
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

    let result = run_batch(client, tasks, progress.clone(), current.clone(), cancel.clone()).await;
    reporter.abort();
    result?;

    // --- Fase D: post-proceso (links + permisos) ---
    let _ = on_event.send(RuntimeEvent::Phase { phase: "finalize".into() });
    for (link, target) in links {
        make_link(&link, &target)?;
    }
    for f in exec_files {
        mark_executable(&f)?;
    }

    // --- Fase E: verificar que el binario quedo disponible ---
    let managed = detect::find_managed_runtime(major)?.ok_or_else(|| {
        AetherError::InvalidState(format!(
            "el runtime java-{major} se descargo pero no se encontro el ejecutable"
        ))
    })?;

    let _ = on_event.send(RuntimeEvent::Done {
        major,
        java_path: managed.java_path.clone(),
    });
    Ok(managed)
}