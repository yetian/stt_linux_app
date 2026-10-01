use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SummaryProvider {
    Ollama,
    Openai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeOptions {
    pub provider: SummaryProvider,
    pub endpoint: String,
    pub model: String,
    pub target_language: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LlmModel {
    pub name: String,
    pub size_bytes: u64,
}

pub fn build_system_prompt(target_language: &str) -> String {
    format!(
        "You are an expert executive assistant. Summarize the following transcript in {target_language}.\n\
         Output structured Markdown containing:\n\
         1. Executive Summary (3-5 sentences)\n\
         2. Key Topics & Discussions (Grouped logically)\n\
         3. Action Items per Speaker (With responsibilities and tasks)"
    )
}

pub async fn summarize(transcript: &str, options: &SummarizeOptions) -> AppResult<String> {
    match options.provider {
        SummaryProvider::Ollama => summarize_with_ollama(transcript, options).await,
        SummaryProvider::Openai => summarize_with_openai(transcript, options).await,
    }
}

fn http_client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(AppError::from)
}

fn normalize(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

async fn summarize_with_ollama(
    transcript: &str,
    options: &SummarizeOptions,
) -> AppResult<String> {
    let url = format!("{}/api/generate", normalize(&options.endpoint));
    let system = build_system_prompt(&options.target_language);

    let payload = OllamaRequest {
        model: &options.model,
        prompt: transcript,
        system: &system,
        stream: false,
    };

    let response = http_client()?
        .post(&url)
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::msg(format!(
            "ollama request failed with status {status}"
        )));
    }

    let body: OllamaResponse = response.json().await?;
    Ok(body.response)
}

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageResponse,
}

#[derive(Deserialize)]
struct OpenAiMessageResponse {
    content: Option<String>,
}

async fn summarize_with_openai(
    transcript: &str,
    options: &SummarizeOptions,
) -> AppResult<String> {
    let url = format!("{}/chat/completions", normalize(&options.endpoint));
    let system = build_system_prompt(&options.target_language);

    let payload = OpenAiRequest {
        model: &options.model,
        messages: vec![
            OpenAiMessage {
                role: "system",
                content: &system,
            },
            OpenAiMessage {
                role: "user",
                content: transcript,
            },
        ],
        stream: false,
    };

    let response = http_client()?
        .post(&url)
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::msg(format!(
            "openai-compatible request failed with status {status}"
        )));
    }

    let body: OpenAiResponse = response.json().await?;
    body.choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| AppError::msg("llm returned an empty response"))
}

#[derive(Deserialize)]
struct OllamaTags {
    models: Vec<OllamaTagModel>,
}

#[derive(Deserialize)]
struct OllamaTagModel {
    name: String,
    #[serde(default)]
    size: u64,
}

pub async fn list_ollama_models(endpoint: &str) -> AppResult<Vec<LlmModel>> {
    let url = format!("{}/api/tags", normalize(endpoint));

    let response = http_client()?.get(&url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::msg(format!(
            "ollama tags request failed with status {status}"
        )));
    }

    let body: OllamaTags = response.json().await?;
    Ok(body
        .models
        .into_iter()
        .map(|model| LlmModel {
            name: model.name,
            size_bytes: model.size,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_includes_target_language_and_structure() {
        let prompt = build_system_prompt("Deutsch");
        assert!(prompt.contains("Deutsch"));
        assert!(prompt.contains("Executive Summary"));
        assert!(prompt.contains("Action Items per Speaker"));
    }

    #[test]
    fn normalizes_endpoint() {
        assert_eq!(normalize("http://localhost:11434/"), "http://localhost:11434");
        assert_eq!(normalize("  http://localhost:1234/v1  "), "http://localhost:1234/v1");
    }

    #[test]
    fn provider_deserializes_lowercase() {
        let ollama: SummaryProvider = serde_json::from_str("\"ollama\"").unwrap();
        let openai: SummaryProvider = serde_json::from_str("\"openai\"").unwrap();
        assert_eq!(ollama, SummaryProvider::Ollama);
        assert_eq!(openai, SummaryProvider::Openai);
    }
}
