use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub recording_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub project_id: Option<String>,
    pub file_name: String,
    pub source_path: String,
    pub audio_duration_secs: Option<f64>,
    pub detected_language: Option<String>,
    pub status: String,
    pub transcript_raw: Option<String>,
    pub summary_markdown: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub recording_id: String,
    pub tag_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRecording {
    pub file_name: String,
    pub source_path: String,
    pub project_id: Option<String>,
    pub audio_duration_secs: Option<f64>,
}
