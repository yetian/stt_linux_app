use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub const PIPELINE_PROGRESS_EVENT: &str = "pipeline-progress";

#[derive(Debug, Clone, Serialize)]
pub struct PipelineProgress {
    pub recording_id: String,
    pub stage: String,
    pub step: Option<String>,
    pub progress: Option<f32>,
    pub done: bool,
}

pub fn emit(
    app: &AppHandle,
    recording_id: &str,
    stage: &str,
    step: Option<&str>,
    progress: Option<f32>,
) {
    let _ = app.emit(
        PIPELINE_PROGRESS_EVENT,
        PipelineProgress {
            recording_id: recording_id.to_string(),
            stage: stage.to_string(),
            step: step.map(|value| value.to_string()),
            progress,
            done: false,
        },
    );
}
