use std::f32::consts::PI;
use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

pub const NUM_MEL_BINS: usize = 80;

const FRAME_MS: usize = 25;
const HOP_MS: usize = 10;
const LOW_FREQ: f32 = 20.0;
const PREEMPHASIS: f32 = 0.97;

fn to_mel(frequency: f32) -> f32 {
    2595.0 * (1.0 + frequency / 700.0).log10()
}

fn to_hz(mel: f32) -> f32 {
    700.0 * (10f32.powf(mel / 2595.0) - 1.0)
}

pub struct Fbank {
    fft: Arc<dyn Fft<f32>>,
    fft_size: usize,
    frame_len: usize,
    hop: usize,
    window: Vec<f32>,
    filters: Vec<Vec<f32>>,
}

impl Fbank {
    pub fn new(sample_rate: u32) -> Self {
        let frame_len = sample_rate as usize * FRAME_MS / 1000;
        let hop = sample_rate as usize * HOP_MS / 1000;
        let fft_size = frame_len.next_power_of_two();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        let window = (0..frame_len)
            .map(|index| {
                0.54 - 0.46 * (2.0 * PI * index as f32 / (frame_len as f32 - 1.0)).cos()
            })
            .collect();

        let filters = mel_filterbank(
            NUM_MEL_BINS,
            fft_size,
            sample_rate,
            LOW_FREQ,
            sample_rate as f32 / 2.0,
        );

        Self {
            fft,
            fft_size,
            frame_len,
            hop,
            window,
            filters,
        }
    }

    pub fn extract(&self, samples: &[f32]) -> Vec<Vec<f32>> {
        if samples.len() < self.frame_len {
            return Vec::new();
        }

        let spectrum_len = self.fft_size / 2 + 1;
        let mut frames: Vec<Vec<f32>> = Vec::new();
        let mut offset = 0;

        while offset + self.frame_len <= samples.len() {
            let frame = &samples[offset..offset + self.frame_len];
            let mut buffer = vec![Complex::new(0.0f32, 0.0f32); self.fft_size];
            let mut previous = if offset == 0 { 0.0 } else { samples[offset - 1] };

            for (index, sample) in frame.iter().enumerate() {
                let emphasized = sample - PREEMPHASIS * previous;
                previous = *sample;
                buffer[index] = Complex::new(emphasized * self.window[index], 0.0);
            }

            self.fft.process(&mut buffer);

            let mut power = vec![0f32; spectrum_len];
            for (index, value) in buffer.iter().take(spectrum_len).enumerate() {
                power[index] = value.norm_sqr();
            }

            let mut features = vec![0f32; NUM_MEL_BINS];
            for (bin, filter) in self.filters.iter().enumerate() {
                let mut sum = 0f32;
                for (index, weight) in filter.iter().enumerate() {
                    sum += weight * power[index];
                }
                features[bin] = (sum + 1e-10).ln();
            }

            frames.push(features);
            offset += self.hop;
        }

        apply_cmn(&mut frames);
        frames
    }
}

fn hz_to_bin(frequency: f32, fft_size: usize, sample_rate: u32) -> usize {
    let bin = ((fft_size as f32 + 1.0) * frequency / sample_rate as f32).floor();
    bin.max(0.0) as usize
}

fn mel_filterbank(
    num_bins: usize,
    fft_size: usize,
    sample_rate: u32,
    low_freq: f32,
    high_freq: f32,
) -> Vec<Vec<f32>> {
    let spectrum_len = fft_size / 2 + 1;
    let low_mel = to_mel(low_freq);
    let high_mel = to_mel(high_freq);

    let points: Vec<f32> = (0..num_bins + 2)
        .map(|index| low_mel + (high_mel - low_mel) * index as f32 / (num_bins + 1) as f32)
        .collect();

    let bins: Vec<usize> = points
        .iter()
        .map(|mel| hz_to_bin(to_hz(*mel), fft_size, sample_rate).min(spectrum_len - 1))
        .collect();

    let mut filters = vec![vec![0f32; spectrum_len]; num_bins];

    for bin in 0..num_bins {
        let left = bins[bin];
        let center = bins[bin + 1];
        let right = bins[bin + 2];

        for index in left..=right.min(spectrum_len - 1) {
            let weight = if index <= center {
                let denominator = (center as f32 - left as f32).max(1.0);
                (index as f32 - left as f32) / denominator
            } else {
                let denominator = (right as f32 - center as f32).max(1.0);
                (right as f32 - index as f32) / denominator
            };
            filters[bin][index] = weight.max(0.0);
        }
    }

    filters
}

fn apply_cmn(frames: &mut [Vec<f32>]) {
    if frames.is_empty() {
        return;
    }

    let dim = frames[0].len();
    let count = frames.len() as f32;
    let mut means = vec![0f32; dim];

    for frame in frames.iter() {
        for (index, value) in frame.iter().enumerate() {
            means[index] += value;
        }
    }

    for mean in means.iter_mut() {
        *mean /= count;
    }

    for frame in frames.iter_mut() {
        for (index, value) in frame.iter_mut().enumerate() {
            *value -= means[index];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(seconds: f32, sample_rate: u32) -> Vec<f32> {
        let count = (sample_rate as f32 * seconds) as usize;
        (0..count)
            .map(|index| {
                (2.0 * std::f32::consts::PI * 160.0 * index as f32 / sample_rate as f32).sin() * 0.5
            })
            .collect()
    }

    #[test]
    fn mel_scale_round_trips() {
        for frequency in [20.0, 440.0, 1_000.0, 4_000.0, 8_000.0] {
            assert!((to_hz(to_mel(frequency)) - frequency).abs() < 0.5);
        }
    }

    #[test]
    fn extracts_frames_with_expected_dimensions() {
        let fbank = Fbank::new(16_000);
        let features = fbank.extract(&sine(1.0, 16_000));

        assert!(!features.is_empty());
        assert!(features.iter().all(|frame| frame.len() == NUM_MEL_BINS));
    }

    #[test]
    fn short_input_returns_no_frames() {
        let fbank = Fbank::new(16_000);
        assert!(fbank.extract(&[0.0f32; 100]).is_empty());
    }
}
