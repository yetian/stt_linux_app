use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::{AppError, AppResult};

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

pub fn decode_to_mono_16k(path: &Path) -> AppResult<Vec<f32>> {
    let file = File::open(path)?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        hint.with_extension(extension);
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        stream,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;

    let (track_id, codec_params) = {
        let track = format
            .default_track()
            .ok_or_else(|| AppError::msg("no audio track found in file"))?;
        (track.id, track.codec_params.clone())
    };

    let mut sample_rate = codec_params.sample_rate.unwrap_or(TARGET_SAMPLE_RATE);
    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default())?;

    let mut samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(error) => return Err(AppError::msg(format!("decode error: {error}"))),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                sample_rate = spec.rate;

                let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                buffer.copy_interleaved_ref(decoded);

                let channels = spec.channels.count().max(1);
                if channels == 1 {
                    samples.extend_from_slice(buffer.samples());
                } else {
                    for frame in buffer.samples().chunks(channels) {
                        let sum: f32 = frame.iter().copied().sum();
                        samples.push(sum / channels as f32);
                    }
                }
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(error) => return Err(AppError::msg(format!("decode error: {error}"))),
        }
    }

    if samples.is_empty() {
        return Err(AppError::msg("no audio samples decoded"));
    }

    if sample_rate != TARGET_SAMPLE_RATE {
        samples = resample_linear(&samples, sample_rate, TARGET_SAMPLE_RATE);
    }

    Ok(samples)
}

fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if input.is_empty() || from_rate == to_rate {
        return input.to_vec();
    }

    let ratio = to_rate as f64 / from_rate as f64;
    let output_len = ((input.len() as f64) * ratio).round() as usize;
    let last = input.len() - 1;
    let mut output = Vec::with_capacity(output_len);

    for index in 0..output_len {
        let position = index as f64 / ratio;
        let base = position.floor() as usize;
        let fraction = (position - base as f64) as f32;
        let left = input[base.min(last)];
        let right = input[(base + 1).min(last)];
        output.push(left + (right - left) * fraction);
    }

    output
}

pub fn probe_duration_secs(path: &Path) -> Option<f64> {
    let file = File::open(path).ok()?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        hint.with_extension(extension);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;

    let track = probed.format.default_track()?;
    let sample_rate = track.codec_params.sample_rate?;
    let frames = track.codec_params.n_frames?;

    if sample_rate == 0 {
        return None;
    }

    Some(frames as f64 / sample_rate as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav_bytes(sample_rate: u32, seconds: u32) -> Vec<u8> {
        let samples = sample_rate * seconds;
        let data_len = samples * 2;
        let mut bytes = Vec::with_capacity(44 + data_len as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        bytes.extend(std::iter::repeat(0u8).take(data_len as usize));
        bytes
    }

    #[test]
    fn probes_wav_duration() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tone.wav");
        std::fs::write(&path, wav_bytes(8_000, 1)).unwrap();

        let duration = probe_duration_secs(&path).unwrap();
        assert!((duration - 1.0).abs() < 0.05, "{duration}");
    }

    #[test]
    fn probes_missing_file_returns_none() {
        assert!(probe_duration_secs(Path::new("/no/such/file.wav")).is_none());
    }

    #[test]
    fn same_rate_is_identity() {
        let input = vec![0.0, 0.5, 1.0, -0.5];
        assert_eq!(resample_linear(&input, 16_000, 16_000), input);
    }

    #[test]
    fn downsamples_halves_length() {
        let input: Vec<f32> = (0..1600).map(|index| index as f32).collect();
        assert_eq!(resample_linear(&input, 16_000, 8_000).len(), 800);
    }

    #[test]
    fn upsamples_doubles_length() {
        let input = vec![0.0f32; 800];
        assert_eq!(resample_linear(&input, 8_000, 16_000).len(), 1600);
    }

    #[test]
    fn empty_input_is_empty() {
        assert!(resample_linear(&[], 16_000, 8_000).is_empty());
    }

    #[test]
    fn interpolates_between_samples() {
        let output = resample_linear(&[0.0, 1.0], 2, 4);
        assert_eq!(output.len(), 4);
        assert!((output[0] - 0.0).abs() < 1e-6);
        assert!((output[1] - 0.5).abs() < 1e-6);
        assert!((output[2] - 1.0).abs() < 1e-6);
        assert!((output[3] - 1.0).abs() < 1e-6);
    }
}
