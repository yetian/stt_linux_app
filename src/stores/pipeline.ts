import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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

const STAGE_SPAN: Record<string, [number, number]> = {
  decoding: [0, 10],
  transcribing: [10, 55],
  clustering: [55, 85],
  summarizing: [85, 100],
};

interface PipelineProgressEvent {
  recording_id: string;
  stage: string;
  step: string | null;
  progress: number | null;
  done: boolean;
}

export const usePipelineStore = defineStore("pipeline", () => {
  const stage = ref<PipelineStage>("idle");
  const step = ref<string | null>(null);
  const progress = ref(0);
  const currentFile = ref<string | null>(null);
  const currentRecordingId = ref<string | null>(null);
  const errorMessage = ref<string | null>(null);
  const startedAt = ref<number | null>(null);
  const elapsedMs = ref(0);

  let timer: ReturnType<typeof setInterval> | undefined;
  let unlisten: UnlistenFn | null = null;

  const activeStepIndex = computed(() => PIPELINE_STAGES.indexOf(stage.value));
  const isRunning = computed(
    () => stage.value !== "idle" && stage.value !== "completed" && stage.value !== "failed",
  );

  const etaMs = computed(() => {
    if (!isRunning.value || startedAt.value === null) return null;
    if (progress.value <= 0 || progress.value >= 100) return null;
    return (elapsedMs.value * (100 - progress.value)) / progress.value;
  });

  function startTimer(): void {
    stopTimer();
    timer = setInterval(() => {
      if (startedAt.value !== null) {
        elapsedMs.value = Date.now() - startedAt.value;
      }
    }, 500);
  }

  function stopTimer(): void {
    if (timer) {
      clearInterval(timer);
      timer = undefined;
    }
  }

  function computeProgress(nextStage: string, fraction: number | null): void {
    const span = STAGE_SPAN[nextStage];
    if (!span) return;
    const [base, end] = span;
    if (fraction === null) {
      progress.value = Math.max(progress.value, base);
    } else {
      progress.value = Math.min(100, Math.max(progress.value, base + fraction * (end - base)));
    }
  }

  async function initialize(): Promise<void> {
    if (unlisten) return;
    unlisten = await listen<PipelineProgressEvent>("pipeline-progress", (event) => {
      const payload = event.payload;
      if (currentRecordingId.value && payload.recording_id !== currentRecordingId.value) return;

      stage.value = payload.stage as PipelineStage;
      step.value = payload.step;
      errorMessage.value = null;
      computeProgress(payload.stage, payload.progress);
      if (startedAt.value === null) {
        startedAt.value = Date.now();
        startTimer();
      }
    });
  }

  function begin(recordingId: string, file: string): void {
    stage.value = "decoding";
    step.value = null;
    progress.value = 0;
    errorMessage.value = null;
    currentFile.value = file;
    currentRecordingId.value = recordingId;
    startedAt.value = Date.now();
    elapsedMs.value = 0;
    startTimer();
  }

  function succeed(): void {
    stopTimer();
    stage.value = "completed";
    step.value = null;
    progress.value = 100;
  }

  function fail(message: string): void {
    stopTimer();
    errorMessage.value = message;
    stage.value = "failed";
  }

  function reset(): void {
    stopTimer();
    stage.value = "idle";
    step.value = null;
    progress.value = 0;
    currentFile.value = null;
    currentRecordingId.value = null;
    errorMessage.value = null;
    startedAt.value = null;
    elapsedMs.value = 0;
  }

  return {
    stage,
    step,
    progress,
    currentFile,
    currentRecordingId,
    errorMessage,
    startedAt,
    elapsedMs,
    etaMs,
    activeStepIndex,
    isRunning,
    initialize,
    begin,
    succeed,
    fail,
    reset,
  };
});
