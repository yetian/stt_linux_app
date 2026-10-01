import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { modelApi } from "@/api/models";
import type { ModelDownloadProgress, ModelStatus } from "@/types";

export const useModelsStore = defineStore("models", () => {
  const models = ref<ModelStatus[]>([]);
  const progress = ref<Record<string, ModelDownloadProgress>>({});
  const error = ref<string | null>(null);

  let unlisten: UnlistenFn | null = null;

  const sttModels = computed(() => models.value.filter((m) => m.kind === "stt"));
  const diarizationModels = computed(() =>
    models.value.filter((m) => m.kind === "diarization"),
  );
  const downloadedStt = computed(() => sttModels.value.find((m) => m.downloaded) ?? null);
  const downloadedDiarization = computed(
    () => diarizationModels.value.find((m) => m.downloaded) ?? null,
  );

  async function initialize(): Promise<void> {
    if (!unlisten) {
      unlisten = await listen<ModelDownloadProgress>(
        "model-download-progress",
        (event) => {
          const payload = event.payload;
          progress.value = { ...progress.value, [payload.filename]: payload };
          if (payload.done) {
            void load();
          }
        },
      );
    }
    await load();
  }

  async function load(): Promise<void> {
    try {
      models.value = await modelApi.list();
    } catch (cause) {
      error.value = String(cause);
    }
  }

  function downloadPercent(filename: string): number {
    const entry = progress.value[filename];
    if (!entry) return 0;
    if (entry.total && entry.total > 0) {
      return Math.min(100, Math.round((entry.downloaded / entry.total) * 100));
    }
    return entry.done ? 100 : 0;
  }

  function isDownloading(filename: string): boolean {
    const entry = progress.value[filename];
    return entry ? !entry.done : false;
  }

  async function download(model: ModelStatus): Promise<void> {
    error.value = null;
    try {
      await modelApi.download(model.url, model.filename);
      await load();
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function remove(model: ModelStatus): Promise<void> {
    error.value = null;
    try {
      await modelApi.remove(model.filename);
      await load();
    } catch (cause) {
      error.value = String(cause);
    }
  }

  return {
    models,
    progress,
    error,
    sttModels,
    diarizationModels,
    downloadedStt,
    downloadedDiarization,
    initialize,
    load,
    download,
    remove,
    downloadPercent,
    isDownloading,
  };
});
