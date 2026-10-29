use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);
pub const DEFAULT_CONTEXT_TOKENS: u32 = 8_192;
const SYSTEM_PROMPT_RESERVE_TOKENS: usize = 1_000;

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
    pub context_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LlmModel {
    pub name: String,
    pub size_bytes: u64,
}

pub fn build_system_prompt(target_language: &str) -> String {
    format!(
        r##"You are an expert meeting analyst and executive assistant. The transcript below may contain several completely different meetings or conversations that are unrelated to each other, and each of those may in turn cover multiple topics. Do NOT assume it is one continuous meeting.

Analyze it and reply ONLY with structured GitHub-flavored Markdown written in {target_language}. The ENTIRE response — section headings, labels, and every sentence — must be in {target_language}; never answer in the transcript's language and never switch languages. Use exactly this structure (translate the headings into {target_language}):

# Summary

## Executive Summary
A concise 3-5 sentence overview of the whole recording, noting how many distinct meetings/topics it appears to contain.

## Meetings & Topics
One subsection per distinct meeting OR topic, in the order they first appear. If the recording contains several separate (possibly unrelated) meetings, create a section for each one. If a single meeting covers several topics, create a subsection per topic. Create as many as needed, and never merge unrelated content into one section.
### <short descriptive meeting or topic title>
- **Overview:** 1-3 sentences on what was discussed.
- **Key points:** important details, decisions, numbers, names, and conclusions.
- **Open questions:** unresolved items, if any.

## Action Items
A single consolidated checklist of every action point across all meetings/topics. Use Markdown task items and include the owner and any due date/timeframe when stated (otherwise "Unassigned" / "None").
- [ ] <action> — Owner: <name or Unassigned> — Due: <date/timeframe or None> — Meeting/Topic: <title>

## Decisions
Bullet list of firm decisions made (omit this section if there were none).

Rules:
- The transcript may combine unrelated meetings with no connection; summarize each one separately and do not treat them as a single conversation.
- Use only information present in the transcript; never invent facts.
- Merge duplicate or repeated points within the same meeting/topic; keep the result concise and skimmable.
- Be concise: one short line per bullet, no filler, no repeated points, and no commentary outside the required sections.
- Always write the output in {target_language}, even if the transcript is in another language. Begin AND end the response in {target_language}; do not drift into another language near the end.
- Keep speaker names if given; otherwise use the "Speaker N" labels from the transcript. Stop after the last required section."##
    )
}

pub fn build_group_system_prompt(target_language: &str) -> String {
    format!(
        r##"You are an expert meeting analyst and executive assistant. You are given the individual summaries (or, when a summary is missing, the transcript) of several recordings that all belong to the same group — typically a single day. The recordings may cover completely unrelated meetings and topics.

Produce ONE consolidated roll-up of the whole group in structured GitHub-flavored Markdown written in {target_language}. The ENTIRE response — section headings, labels, and every sentence — must be in {target_language}; the sources may be in another language, but you must never answer in it or switch languages. Keep it information-dense but concise: include every substantive fact (names, numbers, figures, dates, amounts, decisions and their rationale) using short bullets — no filler, no repetition, no preamble, and no closing remarks. Stop as soon as the required sections are complete.

Use exactly this structure (translate the headings into {target_language}):

# Group Summary

## Overview
A 4-6 sentence synthesis of the group as a whole: the main themes, and how many meetings/topics it covers.

## Meetings & Topics
One detailed section per source recording, in the order given. Capture ALL substantive content from each source — never compress a meeting down to a single line.
### <short descriptive title>
- **Summary:** a 2-4 sentence recap of what the recording was about and its outcome.
- **Key points:** itemized bullets covering the substantive facts: people and their roles, numbers, amounts, dates and deadlines, metrics, examples, arguments, options weighed, decisions and their rationale, and conclusions. Keep each bullet to one short line and state each detail once.
- **Details & context:** background, constraints, dependencies, caveats, and any other supporting information worth preserving (omit if the source has none).
- **Open questions:** unresolved items, risks, and follow-ups (omit if none).

## Consolidated Action Items
Every action point across all recordings, de-duplicated, as Markdown task items with the owner and any due date/timeframe when stated (otherwise "Unassigned" / "None").
- [ ] <action> — Owner: <name or Unassigned> — Due: <date/timeframe or None> — From: <recording title>

## Decisions
Detailed bullet list of firm decisions across the group, each with a short note on the reasoning behind it (omit this section if there were none).

Rules:
- Rely only on the provided summaries/transcripts; never invent facts, but do not omit real details either.
- Preserve specificity: keep exact names, numbers, dates, and figures rather than generalizing them away.
- Merge only items that clearly repeat across meetings; otherwise keep each section self-contained and complete.
- Be concise: one short line per bullet, no filler, no repeated points, no commentary outside the required sections.
- Always write the output in {target_language}, even if the sources are in another language. Begin AND end the response in {target_language}; do not drift into English or any other language near the end.
- Stay organized with headings and bullets so the added detail remains skimmable. Never exceed what is needed to cover the required sections."##
    )
}

pub async fn summarize(transcript: &str, options: &SummarizeOptions) -> AppResult<String> {
    let system = build_system_prompt(&options.target_language);
    summarize_long(&system, transcript, options).await
}

pub async fn summarize_group(document: &str, options: &SummarizeOptions) -> AppResult<String> {
    let system = build_group_system_prompt(&options.target_language);
    summarize_long(&system, document, options).await
}

const MAX_REDUCE_PASSES: usize = 8;

fn output_budget(context_tokens: u32) -> usize {
    ((context_tokens as usize) / 4).clamp(512, 8_192)
}

fn content_budget(context_tokens: u32) -> usize {
    (context_tokens as usize)
        .saturating_sub(output_budget(context_tokens))
        .saturating_sub(SYSTEM_PROMPT_RESERVE_TOKENS)
        .clamp(1_000, 48_000)
}

fn final_notes_budget(context_tokens: u32) -> usize {
    ((context_tokens as usize) / 2)
        .saturating_sub(SYSTEM_PROMPT_RESERVE_TOKENS)
        .max(1_000)
}

fn available_output(context_tokens: u32, input: &str) -> usize {
    (context_tokens as usize)
        .saturating_sub(estimate_tokens(input))
        .saturating_sub(SYSTEM_PROMPT_RESERVE_TOKENS)
        .clamp(512, 16_384)
}

fn take_last_tokens(text: &str, budget: usize) -> String {
    let limit = budget * 4;
    let mut units = 0usize;
    let mut buffer: Vec<char> = Vec::new();
    for character in text.chars().rev() {
        let increment = char_units(character);
        if units + increment > limit {
            break;
        }
        buffer.push(character);
        units += increment;
    }
    buffer.into_iter().rev().collect()
}

fn map_prompt(target_language: &str) -> String {
    format!(
        "You are extracting facts from ONE part of a longer source so the parts can be merged later. Reply ONLY with terse bullet notes written in {target_language}, covering every substantive detail: people and roles, numbers, amounts, dates and deadlines, decisions and their rationale, options considered, and open questions. Write ONLY in {target_language}, even if the source is in another language. Do not write prose, section headings, or a summary; do not omit details; never invent anything. Keep each bullet short."
    )
}

fn reduce_prompt(target_language: &str) -> String {
    format!(
        "You are given several sets of notes that cover consecutive parts of one source. Consolidate them into ONE set of terse bullet notes written in {target_language}: remove duplicates, keep every distinct fact (names, numbers, dates, decisions, rationale), and do not add new information, prose, or headings. Write ONLY in {target_language}."
    )
}

fn char_units(character: char) -> usize {
    if character.is_ascii() {
        1
    } else {
        4
    }
}

fn estimate_tokens(text: &str) -> usize {
    let units: usize = text.chars().map(char_units).sum();
    units.div_ceil(4)
}

fn split_oversized(notes: Vec<String>, budget: usize) -> Vec<String> {
    let mut output = Vec::new();
    for note in notes {
        if estimate_tokens(&note) > budget {
            output.extend(split_chunks(&note, budget));
        } else {
            output.push(note);
        }
    }
    output
}

fn split_hard(text: &str, budget: usize) -> Vec<String> {
    let limit = budget * 4;
    let mut pieces = Vec::new();
    let mut current = String::new();
    let mut units = 0usize;

    for character in text.chars() {
        let increment = char_units(character);
        if units + increment > limit && !current.is_empty() {
            pieces.push(std::mem::take(&mut current));
            units = 0;
        }
        current.push(character);
        units += increment;
    }

    if !current.is_empty() {
        pieces.push(current);
    }
    pieces
}

fn split_chunks(text: &str, budget: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_tokens = 0usize;

    for line in text.split_inclusive('\n') {
        let line_tokens = estimate_tokens(line);
        if line_tokens > budget {
            if !current.trim().is_empty() {
                chunks.push(std::mem::take(&mut current));
                current_tokens = 0;
            }
            chunks.extend(split_hard(line, budget));
            continue;
        }
        if current_tokens + line_tokens > budget && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
            current_tokens = 0;
        }
        current.push_str(line);
        current_tokens += line_tokens;
    }

    if !current.trim().is_empty() {
        chunks.push(current);
    }
    chunks
}

fn batch_by_budget(items: &[String], budget: usize) -> Vec<Vec<String>> {
    let mut batches: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut current_tokens = 0usize;

    for item in items {
        let tokens = estimate_tokens(item);
        if !current.is_empty() && current_tokens + tokens > budget {
            batches.push(std::mem::take(&mut current));
            current_tokens = 0;
        }
        current.push(item.clone());
        current_tokens += tokens;
    }

    if !current.is_empty() {
        batches.push(current);
    }
    batches
}

async fn compress_to_budget(
    mut notes: Vec<String>,
    budget: usize,
    reduce_system: &str,
    options: &SummarizeOptions,
) -> AppResult<String> {
    let mut levels = 0usize;

    loop {
        let joined = notes.join("\n\n");
        if estimate_tokens(&joined) <= budget {
            return Ok(joined);
        }
        if levels >= MAX_REDUCE_PASSES {
            break;
        }

        let expanded = split_oversized(notes, budget);
        let batches = batch_by_budget(&expanded, budget);
        if batches.len() >= expanded.len() {
            break;
        }

        let mut next = Vec::with_capacity(batches.len());
        for batch in batches {
            next.push(generate(reduce_system, &batch.join("\n\n"), options).await?.text);
        }
        notes = next;
        levels += 1;
    }

    let recommended = options.context_tokens.saturating_mul(2).min(131_072);
    Err(AppError::msg(format!(
        "input is too large for the configured context window ({} tokens); increase the model's context window in LM Studio (or the context-window setting) to at least {} tokens and try again",
        options.context_tokens, recommended
    )))
}

async fn finish_summary(
    system: &str,
    user: &str,
    options: &SummarizeOptions,
) -> AppResult<String> {
    let context = options.context_tokens;
    let limit = available_output(context, user);
    let mut result = generate_with_limit(system, user, options, limit).await?;
    let mut output = result.text;
    let mut rounds = 0usize;

    while result.truncated && rounds < 2 {
        rounds += 1;
        let tail = take_last_tokens(&output, (limit / 2).max(256));
        let continuation = format!(
            "Continue the summary exactly where it stopped. Do not repeat any earlier text. Finish the remaining sections and then stop. Write only in {}.\n\n--- text so far ---\n{}",
            options.target_language, tail
        );
        let continuation_limit = available_output(context, &continuation);
        result = generate_with_limit(system, &continuation, options, continuation_limit).await?;
        output.push_str(&result.text);
    }

    if result.truncated {
        return Err(AppError::msg(format!(
            "summary was cut off at the output limit; increase the LLM context window (currently {} tokens) so it has room to finish",
            context
        )));
    }

    Ok(output)
}

async fn summarize_long(
    system: &str,
    user: &str,
    options: &SummarizeOptions,
) -> AppResult<String> {
    let context = options.context_tokens;
    let single_budget = final_notes_budget(context);
    if estimate_tokens(user) <= single_budget {
        return finish_summary(system, user, options).await;
    }

    // Map: compact fact-notes per chunk, already written in the target language.
    let map_system = map_prompt(&options.target_language);
    let mut notes = Vec::new();
    for chunk in split_chunks(user, content_budget(context)) {
        notes.push(generate(&map_system, &chunk, options).await?.text);
    }

    // Reduce to a smaller note set so the final render gets ~half the window for output.
    let reduce_system = reduce_prompt(&options.target_language);
    let merged = compress_to_budget(
        notes,
        final_notes_budget(context),
        &reduce_system,
        options,
    )
    .await?;

    let final_input = format!(
        "Write the final summary in {}. The following are consolidated notes extracted from the complete source; the source may be in another language.\n\n{merged}",
        options.target_language
    );
    finish_summary(system, &final_input, options).await
}

struct Generated {
    text: String,
    truncated: bool,
}

async fn generate(
    system: &str,
    user: &str,
    options: &SummarizeOptions,
) -> AppResult<Generated> {
    let max_tokens = output_budget(options.context_tokens);
    generate_with_limit(system, user, options, max_tokens).await
}

async fn generate_with_limit(
    system: &str,
    user: &str,
    options: &SummarizeOptions,
    max_tokens: usize,
) -> AppResult<Generated> {
    match options.provider {
        SummaryProvider::Ollama => ollama_generate(system, user, options, max_tokens).await,
        SummaryProvider::Openai => openai_chat(system, user, options, max_tokens).await,
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

async fn request_error(response: reqwest::Response, label: &str) -> AppError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let detail: String = body.trim().chars().take(800).collect();
    if detail.is_empty() {
        AppError::msg(format!("{label} request failed with status {status}"))
    } else {
        AppError::msg(format!(
            "{label} request failed with status {status}: {detail}"
        ))
    }
}

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: usize,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: &'a str,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    #[serde(default)]
    done_reason: Option<String>,
}

async fn ollama_generate(
    system: &str,
    user: &str,
    options: &SummarizeOptions,
    max_tokens: usize,
) -> AppResult<Generated> {
    let url = format!("{}/api/generate", normalize(&options.endpoint));

    let payload = OllamaRequest {
        model: &options.model,
        prompt: user,
        system,
        stream: false,
        options: OllamaOptions {
            num_predict: max_tokens,
        },
    };

    let response = http_client()?
        .post(&url)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(request_error(response, "ollama").await);
    }

    let body: OllamaResponse = response.json().await?;
    Ok(Generated {
        text: body.response,
        truncated: body.done_reason.as_deref() == Some("length"),
    })
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
    max_tokens: usize,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageResponse,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiMessageResponse {
    content: Option<String>,
}

async fn openai_chat(
    system: &str,
    user: &str,
    options: &SummarizeOptions,
    max_tokens: usize,
) -> AppResult<Generated> {
    let url = format!("{}/chat/completions", normalize(&options.endpoint));

    let payload = OpenAiRequest {
        model: &options.model,
        messages: vec![
            OpenAiMessage {
                role: "system",
                content: system,
            },
            OpenAiMessage {
                role: "user",
                content: user,
            },
        ],
        stream: false,
        max_tokens,
    };

    let response = http_client()?
        .post(&url)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(request_error(response, "openai-compatible").await);
    }

    let body: OpenAiResponse = response.json().await?;
    let choice = body
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| AppError::msg("llm returned an empty response"))?;
    let text = choice
        .message
        .content
        .ok_or_else(|| AppError::msg("llm returned an empty response"))?;

    Ok(Generated {
        text,
        truncated: choice.finish_reason.as_deref() == Some("length"),
    })
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
        assert!(prompt.contains("Meetings & Topics"));
        assert!(prompt.contains("Action Items"));
        assert!(prompt.contains("Decisions"));
        assert!(prompt.contains("unrelated"));
    }

    #[test]
    fn group_prompt_includes_target_language_and_structure() {
        let prompt = build_group_system_prompt("中文");
        assert!(prompt.contains("中文"));
        assert!(prompt.contains("Overview"));
        assert!(prompt.contains("Consolidated Action Items"));
        assert!(prompt.contains("Meetings & Topics"));
        assert!(prompt.contains("Key points"));
        assert!(prompt.contains("Details & context"));
    }

    #[test]
    fn estimates_tokens_for_ascii_and_non_ascii() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        assert_eq!(estimate_tokens("中文"), 2);
        assert_eq!(estimate_tokens("éé"), 2);
    }

    #[test]
    fn split_chunks_respects_budget_and_keeps_all_text() {
        let line = "0123456789\n";
        let text = line.repeat(2_000);
        let chunks = split_chunks(&text, 100);

        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| estimate_tokens(chunk) <= 100));
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn split_chunks_hard_splits_a_single_long_line() {
        let text = "x".repeat(10_000);
        let chunks = split_chunks(&text, 50);

        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| estimate_tokens(chunk) <= 50));
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn batches_items_within_budget() {
        let items = vec!["a".repeat(40), "b".repeat(40), "c".repeat(40)];
        let batches = batch_by_budget(&items, 20);

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[1].len(), 1);
    }

    #[test]
    fn derives_budgets_from_context_window() {
        assert_eq!(output_budget(8_192), 2_048);
        assert_eq!(output_budget(32_768), 8_192);
        assert_eq!(output_budget(1_000), 512);

        assert_eq!(content_budget(8_192), 5_144);
        assert!(content_budget(32_768) > content_budget(8_192));
        assert_eq!(content_budget(1_000), 1_000);
    }

    #[test]
    fn final_render_budgets_share_the_window() {
        assert_eq!(final_notes_budget(8_192), 3_096);
        assert_eq!(available_output(8_192, &"a".repeat(40)), 7_182);
    }

    #[test]
    fn takes_the_last_tokens_for_continuation() {
        assert_eq!(take_last_tokens("abcdefghij", 1), "ghij");
        assert_eq!(take_last_tokens("hello", 10), "hello");
    }

    #[test]
    fn splits_oversized_notes_without_dropping_text() {
        let notes = vec!["a".repeat(2_000), "b".repeat(10)];
        let split = split_oversized(notes, 50);

        assert!(split.iter().all(|note| estimate_tokens(note) <= 50));
        assert_eq!(split.concat(), format!("{}{}", "a".repeat(2_000), "b".repeat(10)));
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
