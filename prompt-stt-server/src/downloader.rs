use crate::models::{ModelInfo, model_path, models_dir};
use futures_util::StreamExt;
use std::io::Write;
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub done: bool,
    pub error: Option<String>,
}

pub async fn download_model(
    model: ModelInfo,
    progress_tx: watch::Sender<Option<DownloadProgress>>,
) -> Result<(), String> {
    let dir = models_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create dir: {e}"))?;

    let path = model_path(&model);

    let _ = progress_tx.send(Some(DownloadProgress {
        model_id: model.id.clone(),
        downloaded_bytes: 0,
        total_bytes: model.size_mb * 1024 * 1024,
        done: false,
        error: None,
    }));

    let client = reqwest::Client::new();
    let resp = client
        .get(&model.url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    let total = resp.content_length().unwrap_or(model.size_mb * 1024 * 1024);

    let mut file = std::fs::File::create(&path).map_err(|e| format!("Cannot create file: {e}"))?;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {e}"))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Write error: {e}"))?;
        downloaded += chunk.len() as u64;

        let _ = progress_tx.send(Some(DownloadProgress {
            model_id: model.id.clone(),
            downloaded_bytes: downloaded,
            total_bytes: total,
            done: false,
            error: None,
        }));
    }

    let _ = progress_tx.send(Some(DownloadProgress {
        model_id: model.id.clone(),
        downloaded_bytes: total,
        total_bytes: total,
        done: true,
        error: None,
    }));

    Ok(())
}
