import { defineStore } from "pinia";
import { computed, ref } from "vue";

export type PipelineStage =
  | "idle"
  | "decoding"
  | "transcribing"
  | "clustering"
  | "summarizing"
  | "completed"
  | "failed";

export const PIPELINE_STAGES: PipelineStage[] = [
  "decoding",
  "transcribing",
  "clustering",
  "summarizing",
];

export const usePipelineStore = defineStore("pipeline", () => {
  const stage = ref<PipelineStage>("idle");
  const progress = ref(0);
  const currentFile = ref<string | null>(null);
  const errorMessage = ref<string | null>(null);

  const activeStepIndex = computed(() => PIPELINE_STAGES.indexOf(stage.value));
  const isRunning = computed(
    () => stage.value !== "idle" && stage.value !== "completed" && stage.value !== "failed",
  );

  function setStage(next: PipelineStage): void {
    stage.value = next;
    errorMessage.value = null;
    if (next === "completed") {
      progress.value = 100;
    } else if (next === "idle") {
      progress.value = 0;
      currentFile.value = null;
    }
  }

  function setProgress(value: number): void {
    progress.value = Math.min(100, Math.max(0, value));
  }

  function fail(message: string): void {
    errorMessage.value = message;
    stage.value = "failed";
  }

  function reset(): void {
    stage.value = "idle";
    progress.value = 0;
    currentFile.value = null;
    errorMessage.value = null;
  }

  return {
    stage,
    progress,
    currentFile,
    errorMessage,
    activeStepIndex,
    isRunning,
    setStage,
    setProgress,
    fail,
    reset,
  };
});
