<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { DiarizedSegment } from "@/types";

const props = defineProps<{
  transcript?: string | null;
  diarized?: DiarizedSegment[];
}>();

const { t } = useI18n();

const hasDiarized = computed(() => (props.diarized?.length ?? 0) > 0);

const SPEAKER_COLORS = [
  "text-sky-400",
  "text-pink-400",
  "text-lime-400",
  "text-amber-400",
];

function formatTime(milliseconds: number): string {
  const total = Math.floor(milliseconds / 1000);
  const hours = String(Math.floor(total / 3600)).padStart(2, "0");
  const minutes = String(Math.floor((total % 3600) / 60)).padStart(2, "0");
  const seconds = String(total % 60).padStart(2, "0");
  return `${hours}:${minutes}:${seconds}`;
}
</script>

<template>
  <div v-if="hasDiarized" class="flex max-h-[60vh] flex-col gap-3 overflow-y-auto">
    <div
      v-for="(segment, index) in diarized"
      :key="index"
      class="rounded-xl border border-base-800 bg-base-850 px-4 py-3"
    >
      <div class="flex items-center gap-3">
        <span
          class="text-xs font-semibold"
          :class="SPEAKER_COLORS[segment.speaker % SPEAKER_COLORS.length]"
        >
          Speaker {{ segment.speaker }}
        </span>
        <span class="font-mono text-[11px] text-base-500">
          {{ formatTime(segment.start_ms) }}
        </span>
      </div>
      <p class="mt-1.5 text-sm leading-relaxed text-slate-300">{{ segment.text }}</p>
    </div>
  </div>

  <div
    v-else-if="transcript"
    class="max-h-[60vh] overflow-y-auto rounded-xl bg-base-850 p-5 font-mono text-xs leading-relaxed whitespace-pre-wrap text-slate-300"
  >
    {{ transcript }}
  </div>

  <div v-else class="grid place-items-center rounded-2xl border border-base-800 bg-base-900 px-6 py-16 text-center">
    <div class="max-w-sm">
      <div class="mx-auto grid size-12 place-items-center rounded-full bg-base-800 text-base-500">
        <svg viewBox="0 0 24 24" fill="none" class="size-6" stroke="currentColor" stroke-width="1.6">
          <path d="M4 6h16M4 12h10M4 18h13" stroke-linecap="round" />
        </svg>
      </div>
      <p class="mt-4 text-sm font-medium text-slate-300">{{ t("transcript.empty") }}</p>
      <p class="mt-1 text-xs text-base-500">{{ t("transcript.emptyHint") }}</p>
    </div>
  </div>
</template>
