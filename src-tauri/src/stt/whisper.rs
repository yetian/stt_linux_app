use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

pub struct WhisperEngine {
    context: Arc<WhisperContext>,
}

impl WhisperEngine {
    pub fn load(model_path: &Path, use_gpu: bool) -> AppResult<Self> {
        if !model_path.exists() {
            return Err(AppError::msg(format!(
                "whisper model not found at {}",
                model_path.display()
            )));
        }

        let path = model_path
            .to_str()
            .ok_or_else(|| AppError::msg("model path is not valid UTF-8"))?;

        let mut parameters = WhisperContextParameters::default();
        parameters.use_gpu(use_gpu);

        let context = WhisperContext::new_with_params(path, parameters)
            .map_err(|error| AppError::msg(format!("failed to load whisper model: {error}")))?;

        Ok(Self {
            context: Arc::new(context),
        })
    }

    pub fn from_context(context: Arc<WhisperContext>) -> Self {
        Self { context }
    }

    pub fn context(&self) -> Arc<WhisperContext> {
        Arc::clone(&self.context)
    }

    pub fn transcribe<F>(
        &self,
        samples: &[f32],
        language: Option<&str>,
        translate: bool,
        on_progress: F,
    ) -> AppResult<Vec<TranscriptSegment>>
    where
        F: FnMut(i32) + 'static,
    {
        let mut state = self
            .context
            .create_state()
            .map_err(|error| AppError::msg(format!("failed to create whisper state: {error}")))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_translate(translate);
        params.set_language(language);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_n_threads(thread_count());
        params.set_progress_callback_safe(on_progress);

        state
            .full(params, samples)
            .map_err(|error| AppError::msg(format!("transcription failed: {error}")))?;

        let count = state
            .full_n_segments()
            .map_err(|error| AppError::msg(format!("failed to read segments: {error}")))?;

        let mut segments = Vec::with_capacity(count as usize);
        for index in 0..count {
            let text = state
                .full_get_segment_text(index)
                .map_err(|error| AppError::msg(format!("failed to read segment text: {error}")))?;
            let start = state
                .full_get_segment_t0(index)
                .map_err(|error| AppError::msg(format!("failed to read segment start: {error}")))?;
            let end = state
                .full_get_segment_t1(index)
                .map_err(|error| AppError::msg(format!("failed to read segment end: {error}")))?;

            segments.push(TranscriptSegment {
                start_ms: i64::from(start) * 10,
                end_ms: i64::from(end) * 10,
                text: text.trim().to_string(),
            });
        }

        Ok(segments)
    }
}

pub fn format_transcript(segments: &[TranscriptSegment]) -> String {
    let mut output = String::new();

    for segment in segments {
        if segment.text.is_empty() {
            continue;
        }
        output.push_str(&format!(
            "[{}] {}",
            format_timestamp(segment.start_ms),
            segment.text
        ));
        output.push('\n');
    }

    output
}

pub fn format_timestamp(milliseconds: i64) -> String {
    let total_seconds = milliseconds / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub fn parse_transcript(raw: &str) -> Vec<TranscriptSegment> {
    let mut segments: Vec<TranscriptSegment> = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix('[') else {
            continue;
        };
        let Some(close) = rest.find(']') else {
            continue;
        };
        let Some(start_ms) = parse_timestamp(&rest[..close]) else {
            continue;
        };
        let text = strip_speaker_prefix(rest[close + 1..].trim());
        segments.push(TranscriptSegment {
            start_ms,
            end_ms: start_ms,
            text: text.to_string(),
        });
    }

    for index in 0..segments.len() {
        let end = segments
            .get(index + 1)
            .map(|segment| segment.start_ms)
            .unwrap_or(segments[index].start_ms + 5_000);
        segments[index].end_ms = end.max(segments[index].start_ms + 1);
    }

    segments
}

fn parse_timestamp(value: &str) -> Option<i64> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: i64 = parts[0].parse().ok()?;
    let minutes: i64 = parts[1].parse().ok()?;
    let seconds: i64 = parts[2].parse().ok()?;
    Some(((hours * 3600) + (minutes * 60) + seconds) * 1000)
}

fn strip_speaker_prefix(text: &str) -> &str {
    if let Some(rest) = text.strip_prefix("Speaker ") {
        if let Some(colon) = rest.find(':') {
            let label = &rest[..colon];
            if !label.is_empty() && label.trim().chars().all(|c| c.is_ascii_digit()) {
                return rest[colon + 1..].trim();
            }
        }
    }
    text
}

fn thread_count() -> i32 {
    std::thread::available_parallelism()
        .map(|value| value.get() as i32)
        .unwrap_or(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_timestamps() {
        assert_eq!(format_timestamp(0), "00:00:00");
        assert_eq!(format_timestamp(1_000), "00:00:01");
        assert_eq!(format_timestamp(61_000), "00:01:01");
        assert_eq!(format_timestamp(3_661_000), "01:01:01");
    }

    #[test]
    fn formats_transcript_and_skips_empty_segments() {
        let segments = vec![
            TranscriptSegment {
                start_ms: 0,
                end_ms: 1_000,
                text: "hello".to_string(),
            },
            TranscriptSegment {
                start_ms: 1_000,
                end_ms: 2_000,
                text: String::new(),
            },
            TranscriptSegment {
                start_ms: 2_000,
                end_ms: 3_000,
                text: "world".to_string(),
            },
        ];

        assert_eq!(
            format_transcript(&segments),
            "[00:00:00] hello\n[00:00:02] world\n"
        );
    }

    #[test]
    fn formats_empty_transcript() {
        assert_eq!(format_transcript(&[]), "");
    }

    #[test]
    fn parses_formatted_transcript() {
        let raw = "[00:00:00] hello\n[00:00:02] world\n";
        let segments = parse_transcript(raw);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "hello");
        assert_eq!(segments[0].start_ms, 0);
        assert_eq!(segments[0].end_ms, 2_000);
        assert_eq!(segments[1].text, "world");
    }

    #[test]
    fn parses_diarized_transcript_and_strips_speaker() {
        let raw = "[00:01:12] Speaker 0: hello everyone\n[00:01:15] Speaker 1: hi";
        let segments = parse_transcript(raw);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "hello everyone");
        assert_eq!(segments[0].start_ms, 72_000);
        assert_eq!(segments[1].text, "hi");
    }

    #[test]
    fn parses_ignores_malformed_lines() {
        assert!(parse_transcript("no timestamps here\n[bad] nope").is_empty());
    }
}
