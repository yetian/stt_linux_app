use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpeechInterval {
    pub start_ms: i64,
    pub end_ms: i64,
}

const FRAME_MS: usize = 25;
const HOP_MS: usize = 10;
const MIN_SPEECH_MS: i64 = 250;
const MIN_SILENCE_MS: i64 = 200;

pub fn detect_speech(samples: &[f32], sample_rate: u32) -> Vec<SpeechInterval> {
    let frame_len = sample_rate as usize * FRAME_MS / 1000;
    let hop = sample_rate as usize * HOP_MS / 1000;

    if samples.len() < frame_len || hop == 0 {
        return Vec::new();
    }

    let mut energies = Vec::new();
    let mut offset = 0;
    while offset + frame_len <= samples.len() {
        let frame = &samples[offset..offset + frame_len];
        let mean = frame.iter().copied().sum::<f32>() / frame_len as f32;
        let energy = frame
            .iter()
            .map(|sample| (sample - mean).powi(2))
            .sum::<f32>()
            / frame_len as f32;
        energies.push(energy.sqrt());
        offset += hop;
    }

    if energies.is_empty() {
        return Vec::new();
    }

    let mean = energies.iter().sum::<f32>() / energies.len() as f32;
    let variance = energies
        .iter()
        .map(|energy| (energy - mean).powi(2))
        .sum::<f32>()
        / energies.len() as f32;
    let threshold = (mean + 0.5 * variance.sqrt()).max(mean * 0.2).max(1e-4);

    let mut intervals: Vec<SpeechInterval> = Vec::new();
    let mut current: Option<(usize, usize)> = None;

    for (index, energy) in energies.iter().enumerate() {
        if *energy >= threshold {
            match &mut current {
                Some((_, end)) => *end = index,
                None => current = Some((index, index)),
            }
        } else if let Some((start, end)) = current.take() {
            intervals.push(frame_span_to_interval(start, end, hop, frame_len, sample_rate));
        }
    }

    if let Some((start, end)) = current {
        intervals.push(frame_span_to_interval(start, end, hop, frame_len, sample_rate));
    }

    merge_and_filter(intervals)
}

fn frame_span_to_interval(
    start_frame: usize,
    end_frame: usize,
    hop: usize,
    frame_len: usize,
    sample_rate: u32,
) -> SpeechInterval {
    let start_sample = start_frame * hop;
    let end_sample = end_frame * hop + frame_len;
    SpeechInterval {
        start_ms: samples_to_ms(start_sample, sample_rate),
        end_ms: samples_to_ms(end_sample, sample_rate),
    }
}

fn samples_to_ms(samples: usize, sample_rate: u32) -> i64 {
    (samples as i64 * 1000) / sample_rate as i64
}

fn merge_and_filter(intervals: Vec<SpeechInterval>) -> Vec<SpeechInterval> {
    let mut merged: Vec<SpeechInterval> = Vec::new();

    for interval in intervals {
        match merged.last_mut() {
            Some(previous) if interval.start_ms - previous.end_ms <= MIN_SILENCE_MS => {
                previous.end_ms = previous.end_ms.max(interval.end_ms);
            }
            _ => merged.push(interval),
        }
    }

    merged
        .into_iter()
        .filter(|interval| interval.end_ms - interval.start_ms >= MIN_SPEECH_MS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frequency: f32, seconds: f32, sample_rate: u32, amplitude: f32) -> Vec<f32> {
        let count = (sample_rate as f32 * seconds) as usize;
        (0..count)
            .map(|index| {
                amplitude
                    * (2.0 * std::f32::consts::PI * frequency * index as f32 / sample_rate as f32)
                        .sin()
            })
            .collect()
    }

    #[test]
    fn detects_a_single_speech_region() {
        let sample_rate = 16_000;
        let mut samples = vec![0.0f32; sample_rate as usize / 2];
        samples.extend(sine(440.0, 1.0, sample_rate, 0.5));
        samples.extend(vec![0.0f32; sample_rate as usize / 2]);

        let intervals = detect_speech(&samples, sample_rate);

        assert_eq!(intervals.len(), 1);
        let interval = intervals[0];
        assert!(interval.start_ms >= 300 && interval.start_ms <= 800, "{interval:?}");
        assert!(interval.end_ms >= 1_200 && interval.end_ms <= 1_700, "{interval:?}");
    }

    #[test]
    fn silence_has_no_intervals() {
        assert!(detect_speech(&vec![0.0f32; 16_000], 16_000).is_empty());
    }

    #[test]
    fn short_input_returns_empty() {
        assert!(detect_speech(&[0.0f32; 100], 16_000).is_empty());
    }
}
