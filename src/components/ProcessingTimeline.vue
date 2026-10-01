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
</script>

<template>
  <section class="rounded-2xl border border-base-800 bg-base-900 p-5">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-medium text-slate-200">{{ headline }}</h2>
      <span class="font-mono text-[11px] text-base-500">{{ pipeline.progress }}%</span>
    </div>

    <div class="mt-3 h-1.5 w-full overflow-hidden rounded-full bg-base-800">
      <div
        class="h-full rounded-full transition-all duration-500"
        :class="pipeline.stage === 'failed' ? 'bg-rose-500' : 'bg-gradient-to-r from-accent-600 to-accent-400'"
        :style="{ width: `${pipeline.progress}%` }"
      />
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
        <span class="font-mono text-[10px] opacity-70">{{ index + 1 }}/{{ total }}</span>
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
