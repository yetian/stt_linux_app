import { defineStore } from "pinia";
import { computed, nextTick, ref, watch } from "vue";
import { settingsApi, type SettingsMap } from "@/api/settings";
import { i18n, isUiLocale, setUiLocale, type UiLocale } from "@/i18n";
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

const AUDIO_LANGUAGE_VALUES = AUDIO_LANGUAGE_OPTIONS.map((option) => option.value);

const STORAGE_KEY = "lra.settings";

interface SettingsSnapshot {
  uiLocale: UiLocale;
  audioLanguage: AudioLanguage;
  outputLanguage: string;
  summaryProvider: SummaryProvider;
  ollamaEndpoint: string;
  openaiEndpoint: string;
  ollamaModel: string;
  openaiModel: string;
  useGpu: boolean;
}

function serialize(snapshot: SettingsSnapshot): SettingsMap {
  return {
    uiLocale: snapshot.uiLocale,
    audioLanguage: snapshot.audioLanguage,
    outputLanguage: snapshot.outputLanguage,
    summaryProvider: snapshot.summaryProvider,
    ollamaEndpoint: snapshot.ollamaEndpoint,
    openaiEndpoint: snapshot.openaiEndpoint,
    ollamaModel: snapshot.ollamaModel,
    openaiModel: snapshot.openaiModel,
    useGpu: snapshot.useGpu ? "true" : "false",
  };
}

function deserialize(raw: SettingsMap): Partial<SettingsSnapshot> {
  const snapshot: Partial<SettingsSnapshot> = {};

  if (raw.uiLocale !== undefined && isUiLocale(raw.uiLocale)) {
    snapshot.uiLocale = raw.uiLocale;
  }
  if (
    raw.audioLanguage !== undefined &&
    (AUDIO_LANGUAGE_VALUES as string[]).includes(raw.audioLanguage)
  ) {
    snapshot.audioLanguage = raw.audioLanguage as AudioLanguage;
  }
  if (raw.summaryProvider === "ollama" || raw.summaryProvider === "openai") {
    snapshot.summaryProvider = raw.summaryProvider;
  }
  if (raw.useGpu === "true" || raw.useGpu === "false") {
    snapshot.useGpu = raw.useGpu === "true";
  }

  if (raw.outputLanguage !== undefined) snapshot.outputLanguage = raw.outputLanguage;
  if (raw.ollamaEndpoint !== undefined) snapshot.ollamaEndpoint = raw.ollamaEndpoint;
  if (raw.openaiEndpoint !== undefined) snapshot.openaiEndpoint = raw.openaiEndpoint;
  if (raw.ollamaModel !== undefined) snapshot.ollamaModel = raw.ollamaModel;
  if (raw.openaiModel !== undefined) snapshot.openaiModel = raw.openaiModel;

  return snapshot;
}

function readCache(): SettingsMap {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return {};
    return parsed as SettingsMap;
  } catch {
    return {};
  }
}

function writeCache(settings: SettingsMap): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    /* storage full or unavailable — the SQLite copy still holds the values */
  }
}

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

  function collect(): SettingsSnapshot {
    return {
      uiLocale: uiLocale.value,
      audioLanguage: audioLanguage.value,
      outputLanguage: outputLanguage.value,
      summaryProvider: summaryProvider.value,
      ollamaEndpoint: ollamaEndpoint.value,
      openaiEndpoint: openaiEndpoint.value,
      ollamaModel: ollamaModel.value,
      openaiModel: openaiModel.value,
      useGpu: useGpu.value,
    };
  }

  function applySnapshot(snapshot: Partial<SettingsSnapshot>): void {
    if (snapshot.uiLocale !== undefined) {
      uiLocale.value = snapshot.uiLocale;
      setUiLocale(snapshot.uiLocale);
    }
    if (snapshot.audioLanguage !== undefined) audioLanguage.value = snapshot.audioLanguage;
    if (snapshot.summaryProvider !== undefined) summaryProvider.value = snapshot.summaryProvider;
    if (snapshot.ollamaEndpoint !== undefined) ollamaEndpoint.value = snapshot.ollamaEndpoint;
    if (snapshot.openaiEndpoint !== undefined) openaiEndpoint.value = snapshot.openaiEndpoint;
    if (snapshot.ollamaModel !== undefined) ollamaModel.value = snapshot.ollamaModel;
    if (snapshot.openaiModel !== undefined) openaiModel.value = snapshot.openaiModel;
    if (snapshot.useGpu !== undefined) useGpu.value = snapshot.useGpu;
    // last: setUiLocale() above must not clobber a restored output language
    if (snapshot.outputLanguage !== undefined) outputLanguage.value = snapshot.outputLanguage;
  }

  // Restore instantly from the localStorage cache so the UI never starts on
  // hardcoded defaults; SQLite is reconciled afterwards in hydrate().
  applySnapshot(deserialize(readCache()));

  let hydrating = false;
  let userModified = false;

  const snapshotJson = computed(() => JSON.stringify(collect()));

  watch(snapshotJson, () => {
    userModified = true;
    const settings = serialize(collect());
    writeCache(settings);
    if (!hydrating) void settingsApi.save(settings).catch(() => undefined);
  });

  async function hydrate(): Promise<void> {
    hydrating = true;
    userModified = false;
    try {
      const stored = await settingsApi.load();

      // A change landed while loading — the cache is newer, keep it.
      if (userModified) return;

      if (Object.keys(stored).length > 0) {
        applySnapshot(deserialize(stored));
        writeCache(serialize(collect()));
      } else {
        await settingsApi.save(serialize(collect()));
      }

      // Let the watcher flush under `hydrating` so hydration does not re-save.
      await nextTick();
    } catch {
      // SQLite unavailable — the localStorage cache already holds valid values.
    } finally {
      hydrating = false;
    }
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
    hydrate,
  };
});
