<script setup lang="ts">
import { useI18n } from "vue-i18n";
import SidebarSection from "@/components/SidebarSection.vue";
import { useProjectsStore } from "@/stores/projects";
import { useUiStore } from "@/stores/ui";
import type { Recording } from "@/types";

const { t } = useI18n();
const projects = useProjectsStore();
const ui = useUiStore();

const STATUS_COLOR: Record<string, string> = {
  pending: "bg-base-500",
  transcribing: "bg-amber-400",
  summarizing: "bg-sky-400",
  completed: "bg-emerald-400",
  failed: "bg-rose-500",
};

function select(recordingId: string): void {
  projects.setActiveRecording(recordingId);
  ui.setView("workspace");
}

function formatDuration(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds)) return "";
  const total = Math.round(seconds);
  const minutes = Math.floor(total / 60);
  const secs = total % 60;
  return `${minutes}:${String(secs).padStart(2, "0")}`;
}

function formatEpoch(seconds: number | null): string {
  if (seconds === null) return "";
  return new Date(seconds * 1000).toLocaleDateString();
}

function parseSqliteTime(value: string | null): Date | null {
  if (!value) return null;
  const normalized = value.includes("T") ? value : value.replace(" ", "T");
  const date = new Date(`${normalized}Z`);
  return Number.isNaN(date.getTime()) ? null : date;
}

function formatStamp(value: string | null): string {
  const date = parseSqliteTime(value);
  return date ? date.toLocaleDateString() : "";
}

function recordedLine(recording: Recording): string {
  return [formatEpoch(recording.recorded_at), formatDuration(recording.audio_duration_secs)]
    .filter((part) => part !== "")
    .join(" · ");
}
</script>

<template>
  <SidebarSection :title="t('nav.recordings')" icon="fa-headphones">
    <p v-if="projects.recordings.length === 0" class="px-3 py-1.5 text-xs text-base-500">
      {{ t("recordings.empty") }}
    </p>

    <button
      v-for="recording in projects.recordings"
      :key="recording.id"
      type="button"
      class="flex flex-col rounded-lg px-2.5 py-1.5 text-left transition-colors"
      :class="projects.activeRecordingId === recording.id ? 'bg-base-800' : 'hover:bg-base-850'"
      :title="recording.source_path"
      @click="select(recording.id)"
    >
      <span class="flex min-w-0 items-center gap-1.5">
        <span
          class="size-1.5 shrink-0 rounded-full"
          :class="STATUS_COLOR[recording.status] ?? 'bg-base-500'"
        />
        <span class="truncate text-xs text-slate-300">{{ recording.file_name }}</span>
      </span>
      <span class="mt-0.5 flex flex-wrap items-center gap-x-2 pl-3 text-[10px] text-base-500">
        <span>{{ t(`status.${recording.status}`) }}</span>
        <span v-if="recordedLine(recording)">
          <i class="fa-regular fa-calendar mr-1"></i>{{ recordedLine(recording) }}
        </span>
        <span v-if="recording.transcribed_at">
          <i class="fa-solid fa-file-pen mr-1"></i>{{ formatStamp(recording.transcribed_at) }}
        </span>
        <span v-if="recording.summarized_at">
          <i class="fa-solid fa-wand-magic-sparkles mr-1"></i>{{ formatStamp(recording.summarized_at) }}
        </span>
      </span>
    </button>
  </SidebarSection>
</template>
