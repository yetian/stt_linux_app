use std::path::Path;

use ort::ep::CUDA;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;

use crate::diarization::cluster::l2_normalize;
use crate::error::{AppError, AppResult};

fn cuda_session(model_path: &Path) -> Result<Session, String> {
    let builder = Session::builder().map_err(|error| error.to_string())?;
    let builder = builder
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|error| error.to_string())?;
    let mut builder = builder
        .with_execution_providers([CUDA::default().build()])
        .map_err(|error| error.to_string())?;
    builder
        .commit_from_file(model_path)
        .map_err(|error| error.to_string())
}

pub struct SpeakerEmbedder {
    session: Session,
    input_name: String,
}

impl SpeakerEmbedder {
    pub fn load(model_path: &Path, use_gpu: bool) -> AppResult<Self> {
        if !model_path.exists() {
            return Err(AppError::msg(format!(
                "speaker embedding model not found at {}",
                model_path.display()
            )));
        }

        let session = Self::build_session(model_path, use_gpu)?;

        let input_name = session
            .inputs()
            .first()
            .map(|input| input.name().to_string())
            .unwrap_or_else(|| "input".to_string());

        Ok(Self {
            session,
            input_name,
        })
    }

    fn build_session(model_path: &Path, use_gpu: bool) -> AppResult<Session> {
        if use_gpu {
            match cuda_session(model_path) {
                Ok(session) => return Ok(session),
                Err(error) => {
                    eprintln!("CUDA speaker embedder unavailable, falling back to CPU: {error}");
                }
            }
        }

        Session::builder()
            .map_err(|error| AppError::msg(format!("failed to create ort session: {error}")))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|error| AppError::msg(format!("failed to set optimization level: {error}")))?
            .commit_from_file(model_path)
            .map_err(|error| AppError::msg(format!("failed to load speaker model: {error}")))
    }

    pub fn embed(&mut self, features: &[Vec<f32>]) -> AppResult<Vec<f32>> {
        if features.is_empty() {
            return Err(AppError::msg("no features available for embedding"));
        }

        let frames = features.len();
        let dim = features[0].len();
        let mut data = Vec::with_capacity(frames * dim);
        for frame in features {
            data.extend_from_slice(frame);
        }

        let tensor = Tensor::from_array((vec![1i64, frames as i64, dim as i64], data))
            .map_err(|error| AppError::msg(format!("failed to build input tensor: {error}")))?;

        let outputs = self
            .session
            .run(ort::inputs![self.input_name.as_str() => tensor])
            .map_err(|error| AppError::msg(format!("speaker embedding inference failed: {error}")))?;

        let (_, values) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|error| AppError::msg(format!("failed to extract embedding: {error}")))?;

        Ok(l2_normalize(values.to_vec()))
    }
}
