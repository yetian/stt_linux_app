import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { i18n, setUiLocale, type UiLocale } from "@/i18n";
import type { SummaryProvider } from "@/types";

export type AudioLanguage = "auto" | "zh" | "en" | "de" | "fr" | "es" | "ja";

export interface AudioLanguageOption {
  value: AudioLanguage;
  label: string;
}

export const AUDIO_LANGUAGE_OPTIONS: AudioLanguageOption[] = [
  { value: "auto", label: "Auto-detect" },
  { value: "zh", label: "中文 (Chinese)" },
  { value: "en", label: "English" },
  { value: "de", label: "Deutsch" },
  { value: "fr", label: "Français" },
  { value: "es", label: "Español" },
  { value: "ja", label: "日本語" },
];

export const useSettingsStore = defineStore("settings", () => {
  const uiLocale = ref<UiLocale>(i18n.global.locale.value as UiLocale);
  const audioLanguage = ref<AudioLanguage>("auto");
  const outputLanguage = ref<string>(uiLocale.value);

  const summaryProvider = ref<SummaryProvider>("ollama");
  const ollamaEndpoint = ref("http://127.0.0.1:11434");
  const openaiEndpoint = ref("http://127.0.0.1:1234/v1");
  const ollamaModel = ref("qwen2.5:14b");
  const openaiModel = ref("");
  const useGpu = ref(true);

  const activeEndpoint = computed(() =>
    summaryProvider.value === "ollama" ? ollamaEndpoint.value : openaiEndpoint.value,
  );

  const activeModel = computed({
    get: () =>
      summaryProvider.value === "ollama" ? ollamaModel.value : openaiModel.value,
    set: (value: string) => {
      if (summaryProvider.value === "ollama") {
        ollamaModel.value = value;
      } else {
        openaiModel.value = value;
      }
    },
  });

  function setLocale(locale: UiLocale): void {
    uiLocale.value = locale;
    outputLanguage.value = locale;
    setUiLocale(locale);
  }

  function setAudioLanguage(language: AudioLanguage): void {
    audioLanguage.value = language;
  }

  function setOutputLanguage(language: string): void {
    outputLanguage.value = language;
  }

  return {
    uiLocale,
    audioLanguage,
    outputLanguage,
    summaryProvider,
    ollamaEndpoint,
    openaiEndpoint,
    ollamaModel,
    openaiModel,
    useGpu,
    activeEndpoint,
    activeModel,
    setLocale,
    setAudioLanguage,
    setOutputLanguage,
  };
});
