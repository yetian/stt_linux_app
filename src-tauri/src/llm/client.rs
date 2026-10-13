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
        r##"You are an expert meeting analyst and executive assistant. The transcript below may cover one or many distinct topics in any order, and may include multiple speakers.

Analyze it and reply ONLY with structured GitHub-flavored Markdown written in {target_language}. Use exactly this structure:

# Summary

## Executive Summary
A concise 3-5 sentence overview of the entire recording.

## Topics
One subsection per distinct topic, in the order the topics first appear. Create as many as needed.
### <short descriptive topic title>
- **Overview:** 1-3 sentences on what was discussed.
- **Key points:** important details, decisions, numbers, names, and conclusions.
- **Open questions:** unresolved items, if any.

## Action Items
A single consolidated checklist of every action point across all topics. Use Markdown task items and include the owner and any due date/timeframe when stated (otherwise "Unassigned" / "None").
- [ ] <action> — Owner: <name or Unassigned> — Due: <date/timeframe or None> — Topic: <topic>

## Decisions
Bullet list of firm decisions made (omit this section if there were none).

Rules:
- Use only information present in the transcript; never invent facts.
- Merge duplicate or repeated points; keep the result concise and skimmable.
- Always write the output in {target_language}, even if the transcript is in another language.
- Keep speaker names if given; otherwise use the "Speaker N" labels from the transcript."##
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

#[derive(Deserialize)]
struct OpenAiModels {
    data: Vec<OpenAiModelEntry>,
}

#[derive(Deserialize)]
struct OpenAiModelEntry {
    id: String,
}

pub async fn list_openai_models(endpoint: &str) -> AppResult<Vec<LlmModel>> {
    let url = format!("{}/models", normalize(endpoint));

    let response = http_client()?.get(&url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::msg(format!(
            "model list request failed with status {status}"
        )));
    }

    let body: OpenAiModels = response.json().await?;
    Ok(body
        .data
        .into_iter()
        .map(|model| LlmModel {
            name: model.id,
            size_bytes: 0,
        })
        .collect())
}

pub async fn list_models(
    provider: SummaryProvider,
    endpoint: &str,
) -> AppResult<Vec<LlmModel>> {
    match provider {
        SummaryProvider::Ollama => list_ollama_models(endpoint).await,
        SummaryProvider::Openai => list_openai_models(endpoint).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_includes_target_language_and_structure() {
        let prompt = build_system_prompt("Deutsch");
        assert!(prompt.contains("Deutsch"));
        assert!(prompt.contains("Executive Summary"));
        assert!(prompt.contains("Topics"));
        assert!(prompt.contains("Action Items"));
        assert!(prompt.contains("Decisions"));
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
