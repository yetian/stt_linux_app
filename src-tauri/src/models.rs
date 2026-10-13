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
    pub transcript_segments: Option<String>,
    pub recorded_at: Option<i64>,
    pub transcribed_at: Option<String>,
    pub summarized_at: Option<String>,
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
    pub recorded_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_serializes_with_snake_case_keys() {
        let recording = Recording {
            id: "id".to_string(),
            project_id: None,
            file_name: "a.mp3".to_string(),
            source_path: "/x/a.mp3".to_string(),
            audio_duration_secs: Some(1.5),
            detected_language: Some("en".to_string()),
            status: "pending".to_string(),
            transcript_raw: None,
            summary_markdown: None,
            transcript_segments: None,
            recorded_at: Some(1),
            transcribed_at: None,
            summarized_at: None,
            created_at: "now".to_string(),
        };

        let value: serde_json::Value = serde_json::to_value(&recording).unwrap();
        for key in [
            "id",
            "project_id",
            "file_name",
            "source_path",
            "audio_duration_secs",
            "detected_language",
            "status",
            "transcript_raw",
            "summary_markdown",
            "transcript_segments",
            "recorded_at",
            "transcribed_at",
            "summarized_at",
            "created_at",
        ] {
            assert!(value.get(key).is_some(), "missing key: {key}");
        }
    }

    #[test]
    fn new_recording_defaults_optional_fields_when_omitted() {
        let input: NewRecording = serde_json::from_str(
            r#"{"file_name":"a.mp3","source_path":"/x","project_id":null,"audio_duration_secs":null}"#,
        )
        .unwrap();

        assert!(input.recorded_at.is_none());
    }
}
