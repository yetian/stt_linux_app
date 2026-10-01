<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { PIPELINE_STAGES, usePipelineStore, type PipelineStage } from "@/stores/pipeline";

const { t } = useI18n();
const pipeline = usePipelineStore();

const STAGE_KEYS: Record<PipelineStage, string> = {
  idle: "pipeline.idle",
  decoding: "pipeline.decoding",
  transcribing: "pipeline.transcribing",
  clustering: "pipeline.clustering",
  summarizing: "pipeline.summarizing",
  completed: "pipeline.completed",
  failed: "pipeline.failed",
};

const STEP_KEYS: Record<string, string> = {
  vad: "pipeline.stepVad",
  embedding: "pipeline.stepEmbedding",
  clustering: "pipeline.stepClustering",
  connecting: "pipeline.stepConnecting",
  generating: "pipeline.stepGenerating",
};

const total = PIPELINE_STAGES.length;

const currentIndex = computed(() =>
  pipeline.stage === "completed" ? total : pipeline.activeStepIndex,
);

function stepState(index: number): "done" | "active" | "pending" {
  if (pipeline.stage === "completed") return "done";
  if (index < currentIndex.value) return "done";
  if (index === currentIndex.value) return "active";
  return "pending";
}

const headline = computed(() =>
  pipeline.stage === "idle" ? t("pipeline.title") : t(STAGE_KEYS[pipeline.stage]),
);

const stepLabel = computed(() =>
  pipeline.step ? t(STEP_KEYS[pipeline.step] ?? pipeline.step) : "",
);

const showTiming = computed(() => pipeline.isRunning || pipeline.stage === "completed");

function formatDuration(milliseconds: number): string {
  const seconds = Math.max(0, Math.floor(milliseconds / 1000));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
  }
  return `${minutes}:${String(secs).padStart(2, "0")}`;
}
</script>

<template>
  <section class="rounded-2xl border border-base-800 bg-base-900 p-5">
    <div class="flex items-center justify-between gap-2">
      <div class="flex min-w-0 items-center gap-2">
        <h2 class="truncate text-sm font-medium text-slate-200">{{ headline }}</h2>
        <span v-if="stepLabel && pipeline.isRunning" class="truncate text-[11px] text-accent-300">
          · {{ stepLabel }}
        </span>
      </div>
      <span class="shrink-0 font-mono text-[11px] text-base-500">{{ Math.round(pipeline.progress) }}%</span>
    </div>

    <div class="mt-3 h-1.5 w-full overflow-hidden rounded-full bg-base-800">
      <div
        class="h-full rounded-full transition-all duration-500"
        :class="
          pipeline.stage === 'failed'
            ? 'bg-rose-500'
            : pipeline.isRunning && pipeline.progress === 0
              ? 'animate-pulse bg-accent-500/60'
              : 'bg-gradient-to-r from-accent-600 to-accent-400'
        "
        :style="{ width: `${pipeline.progress}%` }"
      />
    </div>

    <div v-if="showTiming" class="mt-2 flex items-center justify-between text-[11px] text-base-500">
      <span>
        <i class="fa-regular fa-clock mr-1"></i>{{ t("pipeline.elapsed") }}
        {{ formatDuration(pipeline.elapsedMs) }}
      </span>
      <span v-if="pipeline.etaMs !== null">
        <i class="fa-solid fa-hourglass-half mr-1"></i>{{ t("pipeline.eta") }}
        {{ formatDuration(pipeline.etaMs) }}
      </span>
      <span v-else-if="pipeline.isRunning">
        <i class="fa-solid fa-hourglass-half mr-1"></i>{{ t("pipeline.eta") }}
        {{ t("pipeline.calculating") }}
      </span>
    </div>

    <ol class="mt-4 grid grid-cols-2 gap-2 sm:grid-cols-4">
      <li
        v-for="(stage, index) in PIPELINE_STAGES"
        :key="stage"
        class="flex items-center gap-2 rounded-lg border px-3 py-2 text-xs transition-colors"
        :class="{
          'border-accent-500/40 bg-accent-500/10 text-accent-300': stepState(index) === 'active',
          'border-base-700 bg-base-850 text-slate-300': stepState(index) === 'done',
          'border-base-800 bg-base-900 text-base-500': stepState(index) === 'pending',
        }"
      >
        <i
          class="fa-solid"
          :class="stepState(index) === 'done' ? 'fa-circle-check' : stepState(index) === 'active' ? 'fa-spinner fa-spin' : 'fa-circle'"
        ></i>
        <span class="truncate">{{ t(STAGE_KEYS[stage]) }}</span>
      </li>
    </ol>

    <p v-if="pipeline.currentFile" class="mt-3 truncate font-mono text-[11px] text-base-500">
      {{ pipeline.currentFile }}
    </p>
    <p v-if="pipeline.errorMessage" class="mt-3 text-xs text-rose-400">
      {{ pipeline.errorMessage }}
    </p>
  </section>
</template>
