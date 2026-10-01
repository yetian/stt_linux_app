use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppResult};
use crate::paths;

const CATALOG: &str = include_str!("../../models.json");
pub const MODEL_DOWNLOAD_PROGRESS_EVENT: &str = "model-download-progress";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub size_mb: u64,
    pub url: String,
    pub filename: String,
}

#[derive(Debug, Deserialize)]
struct Catalog {
    #[serde(default)]
    stt_models: Vec<ModelEntry>,
    #[serde(default)]
    diarization_models: Vec<ModelEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub size_mb: u64,
    pub url: String,
    pub filename: String,
    pub downloaded: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub filename: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub done: bool,
}

fn catalog() -> Catalog {
    serde_json::from_str(CATALOG).unwrap_or_else(|_| Catalog {
        stt_models: Vec::new(),
        diarization_models: Vec::new(),
    })
}

pub fn list_models() -> Vec<ModelStatus> {
    let catalog = catalog();
    let dir = paths::models_dir();
    let mut models = Vec::new();

    for (kind, entries) in [
        ("stt", catalog.stt_models),
        ("diarization", catalog.diarization_models),
    ] {
        for entry in entries {
            let path = dir.join(&entry.filename);
            models.push(ModelStatus {
                id: entry.id,
                name: entry.name,
                kind: kind.to_string(),
                size_mb: entry.size_mb,
                url: entry.url,
                filename: entry.filename,
                downloaded: path.exists(),
                path: path.to_string_lossy().into_owned(),
            });
        }
    }

    models
}

pub async fn download(app: &AppHandle, url: &str, filename: &str) -> AppResult<String> {
    let dir = paths::models_dir();
    std::fs::create_dir_all(&dir)?;

    let destination = dir.join(filename);
    let partial = dir.join(format!("{filename}.part"));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3600))
        .build()?;

    let mut response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(AppError::msg(format!(
            "download failed with status {}",
            response.status()
        )));
    }

    let total = response.content_length();
    let mut file = tokio::fs::File::create(&partial).await?;
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now();

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if last_emit.elapsed() >= Duration::from_millis(200) {
            let _ = app.emit(
                MODEL_DOWNLOAD_PROGRESS_EVENT,
                DownloadProgress {
                    filename: filename.to_string(),
                    downloaded,
                    total,
                    done: false,
                },
            );
            last_emit = Instant::now();
        }
    }

    file.flush().await?;
    drop(file);

    tokio::fs::rename(&partial, &destination).await?;

    let _ = app.emit(
        MODEL_DOWNLOAD_PROGRESS_EVENT,
        DownloadProgress {
            filename: filename.to_string(),
            downloaded,
            total,
            done: true,
        },
    );

    Ok(destination.to_string_lossy().into_owned())
}

pub fn delete_model(filename: &str) -> AppResult<()> {
    let path = paths::models_dir().join(filename);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_stt_and_diarization_models() {
        let catalog = catalog();
        assert!(!catalog.stt_models.is_empty());
        assert!(!catalog.diarization_models.is_empty());
    }

    #[test]
    fn list_models_exposes_kinds_filenames_and_paths() {
        let models = list_models();
        assert!(models.iter().any(|model| model.kind == "stt"));
        assert!(models.iter().any(|model| model.kind == "diarization"));
        assert!(models
            .iter()
            .any(|model| model.filename == "camplusplus.onnx"));
        assert!(models
            .iter()
            .all(|model| !model.name.is_empty() && !model.url.is_empty() && !model.path.is_empty()));
    }

    #[test]
    fn deleting_missing_model_is_ok() {
        assert!(delete_model("definitely-not-a-real-model.bin").is_ok());
    }
}
