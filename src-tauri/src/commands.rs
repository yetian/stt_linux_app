use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;
use tauri::{AppHandle, State};

use crate::audio::decoder;
use crate::db::project_manager as pm;
use crate::diarization::{self, DiarizedSegment, SpeakerTurn};
use crate::error::{AppError, AppResult};
use crate::llm::{self, SummarizeOptions, SummaryProvider};
use crate::models::{NewRecording, Project, Recording, Tag};
use crate::services::model_downloader::{self, ModelStatus};
use crate::stt::{self, TranscriptSegment, WhisperEngine};

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn connection(&self) -> MutexGuard<'_, Connection> {
        self.db.lock().expect("database mutex poisoned")
    }
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> AppResult<Project> {
    let conn = state.connection();
    pm::create_project(&conn, &name, description.as_deref())
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> AppResult<Vec<Project>> {
    let conn = state.connection();
    pm::list_projects(&conn)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, project_id: String) -> AppResult<()> {
    let conn = state.connection();
    pm::delete_project(&conn, &project_id)
}

#[tauri::command]
pub fn add_recording(state: State<'_, AppState>, input: NewRecording) -> AppResult<Recording> {
    let conn = state.connection();
    pm::add_recording(&conn, &input)
}

#[tauri::command]
pub fn list_recordings(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> AppResult<Vec<Recording>> {
    let conn = state.connection();
    pm::list_recordings(&conn, project_id.as_deref())
}

#[tauri::command]
pub fn search_recordings(
    state: State<'_, AppState>,
    query_keyword: Option<String>,
    project_id: Option<String>,
    tag_filter: Option<String>,
) -> AppResult<Vec<Recording>> {
    let conn = state.connection();
    pm::search_recordings(
        &conn,
        query_keyword.as_deref(),
        project_id.as_deref(),
        tag_filter.as_deref(),
    )
}

#[tauri::command]
pub fn assign_recording_to_project(
    state: State<'_, AppState>,
    recording_id: String,
    project_id: Option<String>,
) -> AppResult<()> {
    let conn = state.connection();
    pm::assign_recording_to_project(&conn, &recording_id, project_id.as_deref())
}

#[tauri::command]
pub fn delete_recording(state: State<'_, AppState>, recording_id: String) -> AppResult<()> {
    let conn = state.connection();
    pm::delete_recording(&conn, &recording_id)
}

#[tauri::command]
pub fn update_recording_status(
    state: State<'_, AppState>,
    recording_id: String,
    status: String,
) -> AppResult<()> {
    let conn = state.connection();
    pm::update_recording_status(&conn, &recording_id, &status)
}

#[tauri::command]
pub fn update_recording_metadata(
    state: State<'_, AppState>,
    recording_id: String,
    duration_secs: Option<f64>,
    detected_language: Option<String>,
) -> AppResult<()> {
    let conn = state.connection();
    pm::update_recording_metadata(
        &conn,
        &recording_id,
        duration_secs,
        detected_language.as_deref(),
    )
}

#[tauri::command]
pub fn save_transcript(
    state: State<'_, AppState>,
    recording_id: String,
    transcript: String,
) -> AppResult<()> {
    let conn = state.connection();
    pm::save_transcript(&conn, &recording_id, &transcript)
}

#[tauri::command]
pub fn save_summary(
    state: State<'_, AppState>,
    recording_id: String,
    summary: String,
) -> AppResult<()> {
    let conn = state.connection();
    pm::save_summary(&conn, &recording_id, &summary)
}

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>, recording_id: String) -> AppResult<Vec<Tag>> {
    let conn = state.connection();
    pm::list_tags(&conn, &recording_id)
}

#[tauri::command]
pub fn add_tag(
    state: State<'_, AppState>,
    recording_id: String,
    tag_name: String,
) -> AppResult<()> {
    let conn = state.connection();
    pm::add_tag(&conn, &recording_id, &tag_name)
}

#[tauri::command]
pub fn delete_tag(state: State<'_, AppState>, tag_id: i64) -> AppResult<()> {
    let conn = state.connection();
    pm::delete_tag(&conn, tag_id)
}

#[tauri::command]
pub fn rename_project(
    state: State<'_, AppState>,
    project_id: String,
    name: String,
) -> AppResult<Project> {
    let conn = state.connection();
    pm::rename_project(&conn, &project_id, &name)
}

#[tauri::command]
pub fn rename_recording(
    state: State<'_, AppState>,
    recording_id: String,
    file_name: String,
) -> AppResult<Recording> {
    let conn = state.connection();
    pm::rename_recording(&conn, &recording_id, &file_name)
}

#[tauri::command]
pub async fn summarize_text(
    transcript: String,
    provider: SummaryProvider,
    endpoint: String,
    model: String,
    target_language: String,
) -> AppResult<String> {
    let options = SummarizeOptions {
        provider,
        endpoint,
        model,
        target_language,
    };
    llm::summarize(&transcript, &options).await
}

#[tauri::command]
pub async fn summarize_recording(
    state: State<'_, AppState>,
    recording_id: String,
    provider: SummaryProvider,
    endpoint: String,
    model: String,
    target_language: String,
) -> AppResult<String> {
    let transcript = {
        let conn = state.connection();
        let recording = pm::get_recording(&conn, &recording_id)?;
        recording
            .transcript_raw
            .ok_or_else(|| AppError::msg("recording has no transcript"))?
    };

    let options = SummarizeOptions {
        provider,
        endpoint,
        model,
        target_language,
    };
    let summary = llm::summarize(&transcript, &options).await?;

    {
        let conn = state.connection();
        pm::save_summary(&conn, &recording_id, &summary)?;
        pm::update_recording_status(&conn, &recording_id, "completed")?;
    }

    Ok(summary)
}

#[tauri::command]
pub async fn list_llm_models(endpoint: String) -> AppResult<Vec<llm::client::LlmModel>> {
    llm::client::list_ollama_models(&endpoint).await
}

#[tauri::command]
pub async fn transcribe_recording(
    state: State<'_, AppState>,
    recording_id: String,
    model_path: String,
    language: Option<String>,
    translate: Option<bool>,
) -> AppResult<Vec<TranscriptSegment>> {
    let source_path = {
        let conn = state.connection();
        pm::get_recording(&conn, &recording_id)?.source_path
    };

    {
        let conn = state.connection();
        pm::update_recording_status(&conn, &recording_id, "transcribing")?;
    }

    let translate = translate.unwrap_or(false);
    let language = language.filter(|value| !value.trim().is_empty() && value != "auto");

    let task = tauri::async_runtime::spawn_blocking(
        move || -> AppResult<(Vec<TranscriptSegment>, f64, String)> {
            let samples = decoder::decode_to_mono_16k(Path::new(&source_path))?;
            let duration = samples.len() as f64 / f64::from(decoder::TARGET_SAMPLE_RATE);

            let engine = WhisperEngine::load(Path::new(&model_path))?;
            let segments = engine.transcribe(&samples, language.as_deref(), translate)?;
            let transcript = stt::format_transcript(&segments);

            Ok((segments, duration, transcript))
        },
    )
    .await
    .map_err(|error| AppError::msg(format!("transcription task failed: {error}")))?;

    let (segments, duration, transcript) = match task {
        Ok(value) => value,
        Err(error) => {
            let conn = state.connection();
            let _ = pm::update_recording_status(&conn, &recording_id, "failed");
            return Err(error);
        }
    };

    let segments_json = serde_json::to_string(&segments)
        .map_err(|error| AppError::msg(format!("failed to serialize segments: {error}")))?;

    {
        let conn = state.connection();
        pm::update_recording_metadata(&conn, &recording_id, Some(duration), None)?;
        pm::save_transcript(&conn, &recording_id, &transcript)?;
        pm::save_transcript_segments(&conn, &recording_id, &segments_json)?;
        pm::update_recording_status(&conn, &recording_id, "pending")?;
    }

    Ok(segments)
}

#[tauri::command]
pub async fn diarize_recording(
    state: State<'_, AppState>,
    recording_id: String,
    embedding_model_path: String,
    segments: Option<Vec<TranscriptSegment>>,
    threshold: Option<f32>,
) -> AppResult<Vec<DiarizedSegment>> {
    let (source_path, stored_segments) = {
        let conn = state.connection();
        let recording = pm::get_recording(&conn, &recording_id)?;
        (recording.source_path, recording.transcript_segments)
    };

    let segments = match segments {
        Some(provided) if !provided.is_empty() => provided,
        _ => stored_segments
            .as_deref()
            .and_then(|json| serde_json::from_str::<Vec<TranscriptSegment>>(json).ok())
            .unwrap_or_default(),
    };

    let threshold = threshold.unwrap_or(0.35);

    let task = tauri::async_runtime::spawn_blocking(move || -> AppResult<Vec<SpeakerTurn>> {
        let samples = decoder::decode_to_mono_16k(Path::new(&source_path))?;
        diarization::diarize(
            &samples,
            decoder::TARGET_SAMPLE_RATE,
            Path::new(&embedding_model_path),
            threshold,
        )
    })
    .await
    .map_err(|error| AppError::msg(format!("diarization task failed: {error}")))?;

    let turns = task?;
    let aligned = diarization::align(&segments, &turns);

    if aligned.iter().any(|segment| !segment.text.is_empty()) {
        let formatted = diarization::format_diarized(&aligned);
        let conn = state.connection();
        pm::save_transcript(&conn, &recording_id, &formatted)?;
    }

    Ok(aligned)
}

#[tauri::command]
pub fn list_models() -> Vec<ModelStatus> {
    model_downloader::list_models()
}

#[tauri::command]
pub async fn download_model(app: AppHandle, url: String, filename: String) -> AppResult<String> {
    model_downloader::download(&app, &url, &filename).await
}

#[tauri::command]
pub fn delete_model(filename: String) -> AppResult<()> {
    model_downloader::delete_model(&filename)
}

#[tauri::command]
pub fn export_recording(state: State<'_, AppState>, recording_id: String) -> AppResult<String> {
    let recording = {
        let conn = state.connection();
        pm::get_recording(&conn, &recording_id)?
    };

    let directory = crate::paths::outputs_dir();
    std::fs::create_dir_all(&directory)?;

    let file_path = directory.join(format!("{}.md", sanitize_stem(&recording.file_name)));

    let mut content = String::new();
    content.push_str(&format!("# {}\n\n", recording.file_name));
    if let Some(language) = &recording.detected_language {
        content.push_str(&format!("- **Language:** {language}\n"));
    }
    if let Some(duration) = recording.audio_duration_secs {
        content.push_str(&format!("- **Duration:** {duration:.1}s\n"));
    }
    content.push_str(&format!("- **Status:** {}\n\n", recording.status));

    if let Some(summary) = &recording.summary_markdown {
        content.push_str("## Summary\n\n");
        content.push_str(summary);
        content.push_str("\n\n");
    }
    if let Some(transcript) = &recording.transcript_raw {
        content.push_str("## Transcript\n\n");
        content.push_str(transcript);
        content.push('\n');
    }

    std::fs::write(&file_path, content)?;
    Ok(file_path.to_string_lossy().into_owned())
}

fn sanitize_stem(file_name: &str) -> String {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("recording");

    let sanitized: String = stem
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '-' || character == '_' || character == ' '
            {
                character
            } else {
                '_'
            }
        })
        .collect();

    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "recording".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_safe_characters_and_drops_extension() {
        assert_eq!(sanitize_stem("meeting 1.mp3"), "meeting 1");
        assert_eq!(sanitize_stem("2024-01-02_notes.wav"), "2024-01-02_notes");
    }

    #[test]
    fn replaces_path_and_reserved_characters() {
        assert_eq!(sanitize_stem("a/b:c*.wav"), "b_c_");
    }

    #[test]
    fn falls_back_for_empty_names() {
        assert_eq!(sanitize_stem(""), "recording");
        assert_eq!(sanitize_stem("@@@.mp3"), "___");
    }
}
