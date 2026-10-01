use std::path::Path;

use serde::Serialize;

use crate::diarization::cluster;
use crate::diarization::embedding::SpeakerEmbedder;
use crate::diarization::features::Fbank;
use crate::diarization::vad;
use crate::error::AppResult;
use crate::stt::TranscriptSegment;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SpeakerTurn {
    pub start_ms: i64,
    pub end_ms: i64,
    pub speaker: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiarizedSegment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub speaker: usize,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub enum DiarizationPhase {
    Vad,
    Embedding,
    Clustering,
}

pub fn diarize<F>(
    samples: &[f32],
    sample_rate: u32,
    model_path: &Path,
    threshold: f32,
    use_gpu: bool,
    mut on_progress: F,
) -> AppResult<Vec<SpeakerTurn>>
where
    F: FnMut(DiarizationPhase, f32),
{
    on_progress(DiarizationPhase::Vad, 0.0);
    let intervals = vad::detect_speech(samples, sample_rate);
    on_progress(DiarizationPhase::Vad, 1.0);

    if intervals.is_empty() {
        return Ok(Vec::new());
    }

    let fbank = Fbank::new(sample_rate);
    let mut embedder = SpeakerEmbedder::load(model_path, use_gpu)?;

    let total = intervals.len();
    let mut embeddings = Vec::new();
    let mut used_intervals = Vec::new();

    for (index, interval) in intervals.into_iter().enumerate() {
        let start = ms_to_sample(interval.start_ms, sample_rate);
        let end = ms_to_sample(interval.end_ms, sample_rate).min(samples.len());
        if end > start {
            let features = fbank.extract(&samples[start..end]);
            if !features.is_empty() {
                embeddings.push(embedder.embed(&features)?);
                used_intervals.push(interval);
            }
        }

        on_progress(
            DiarizationPhase::Embedding,
            (index + 1) as f32 / total as f32,
        );
    }

    if embeddings.is_empty() {
        return Ok(Vec::new());
    }

    on_progress(DiarizationPhase::Clustering, 0.0);
    let labels = cluster::cluster(&embeddings, threshold);
    on_progress(DiarizationPhase::Clustering, 1.0);

    Ok(used_intervals
        .into_iter()
        .zip(labels)
        .map(|(interval, label)| SpeakerTurn {
            start_ms: interval.start_ms,
            end_ms: interval.end_ms,
            speaker: label,
        })
        .collect())
}

pub fn align(segments: &[TranscriptSegment], turns: &[SpeakerTurn]) -> Vec<DiarizedSegment> {
    if segments.is_empty() {
        return turns
            .iter()
            .map(|turn| DiarizedSegment {
                start_ms: turn.start_ms,
                end_ms: turn.end_ms,
                speaker: turn.speaker,
                text: String::new(),
            })
            .collect();
    }

    segments
        .iter()
        .map(|segment| {
            let speaker = best_speaker(segment.start_ms, segment.end_ms, turns);
            DiarizedSegment {
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
                speaker,
                text: segment.text.clone(),
            }
        })
        .collect()
}

pub fn format_diarized(segments: &[DiarizedSegment]) -> String {
    let mut output = String::new();

    for segment in segments {
        if segment.text.is_empty() {
            continue;
        }
        output.push_str(&format!(
            "[{}] Speaker {}: {}\n",
            crate::stt::whisper::format_timestamp(segment.start_ms),
            segment.speaker,
            segment.text
        ));
    }

    output
}

fn best_speaker(start_ms: i64, end_ms: i64, turns: &[SpeakerTurn]) -> usize {
    let mut best_speaker = 0usize;
    let mut best_overlap = 0i64;

    for turn in turns {
        let overlap = (end_ms.min(turn.end_ms) - start_ms.max(turn.start_ms)).max(0);
        if overlap > best_overlap {
            best_overlap = overlap;
            best_speaker = turn.speaker;
        }
    }

    best_speaker
}

fn ms_to_sample(milliseconds: i64, sample_rate: u32) -> usize {
    let samples = milliseconds.max(0) as i64 * sample_rate as i64 / 1000;
    samples as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(start_ms: i64, end_ms: i64, text: &str) -> TranscriptSegment {
        TranscriptSegment {
            start_ms,
            end_ms,
            text: text.to_string(),
        }
    }

    #[test]
    fn converts_milliseconds_to_samples() {
        assert_eq!(ms_to_sample(0, 16_000), 0);
        assert_eq!(ms_to_sample(1_000, 16_000), 16_000);
        assert_eq!(ms_to_sample(500, 16_000), 8_000);
        assert_eq!(ms_to_sample(-5, 16_000), 0);
    }

    #[test]
    fn aligns_segments_to_dominant_speaker() {
        let segments = vec![segment(0, 1_000, "a"), segment(1_000, 2_000, "b")];
        let turns = vec![
            SpeakerTurn {
                start_ms: 0,
                end_ms: 900,
                speaker: 0,
            },
            SpeakerTurn {
                start_ms: 1_000,
                end_ms: 2_000,
                speaker: 1,
            },
        ];

        let aligned = align(&segments, &turns);

        assert_eq!(aligned[0].speaker, 0);
        assert_eq!(aligned[1].speaker, 1);
        assert_eq!(aligned[0].text, "a");
    }

    #[test]
    fn alignment_without_segments_uses_turns() {
        let turns = vec![SpeakerTurn {
            start_ms: 0,
            end_ms: 1_000,
            speaker: 3,
        }];
        let aligned = align(&[], &turns);
        assert_eq!(aligned.len(), 1);
        assert_eq!(aligned[0].speaker, 3);
        assert!(aligned[0].text.is_empty());
    }

    #[test]
    fn alignment_without_turns_defaults_to_speaker_zero() {
        let segments = vec![segment(0, 1_000, "x")];
        assert_eq!(align(&segments, &[])[0].speaker, 0);
    }

    #[test]
    fn picks_longest_overlapping_turn() {
        let segment = segment(0, 1_000, "x");
        let turns = vec![
            SpeakerTurn {
                start_ms: 0,
                end_ms: 200,
                speaker: 4,
            },
            SpeakerTurn {
                start_ms: 200,
                end_ms: 1_000,
                speaker: 7,
            },
        ];
        assert_eq!(align(&[segment], &turns)[0].speaker, 7);
    }

    #[test]
    fn formats_diarized_lines() {
        let segments = vec![DiarizedSegment {
            start_ms: 0,
            end_ms: 1_000,
            speaker: 2,
            text: "hi".to_string(),
        }];
        assert_eq!(format_diarized(&segments), "[00:00:00] Speaker 2: hi\n");
    }
}
