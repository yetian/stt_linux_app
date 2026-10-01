import { invoke } from "@tauri-apps/api/core";
import type { LlmModel, SummaryProvider } from "@/types";

export interface SummarizeParams {
  provider: SummaryProvider;
  endpoint: string;
  model: string;
  targetLanguage: string;
}

export const llmApi = {
  summarizeText: (transcript: string, params: SummarizeParams) =>
    invoke<string>("summarize_text", {
      transcript,
      provider: params.provider,
      endpoint: params.endpoint,
      model: params.model,
      targetLanguage: params.targetLanguage,
    }),

  summarizeRecording: (recordingId: string, params: SummarizeParams) =>
    invoke<string>("summarize_recording", {
      recordingId,
      provider: params.provider,
      endpoint: params.endpoint,
      model: params.model,
      targetLanguage: params.targetLanguage,
    }),

  listModels: (provider: SummaryProvider, endpoint: string) =>
    invoke<LlmModel[]>("list_llm_models", { provider, endpoint }),
};
