//! Pool de descargas concurrente sobre Tokio. Streaming a disco,
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
                path: task.dest.to_string_lossy().to_string(),
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
