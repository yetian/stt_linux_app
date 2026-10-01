use std::path::Path;

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
    context: WhisperContext,
}

impl WhisperEngine {
    pub fn load(model_path: &Path) -> AppResult<Self> {
        if !model_path.exists() {
            return Err(AppError::msg(format!(
                "whisper model not found at {}",
                model_path.display()
            )));
        }

        let model_path = model_path
            .to_str()
            .ok_or_else(|| AppError::msg("model path is not valid UTF-8"))?;

        let context = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        )
        .map_err(|error| AppError::msg(format!("failed to load whisper model: {error}")))?;

        Ok(Self { context })
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
}
