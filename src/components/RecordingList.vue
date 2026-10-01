<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useProjectsStore } from "@/stores/projects";
import { useUiStore } from "@/stores/ui";

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
</script>

<template>
  <div class="flex flex-col gap-1">
    <span class="px-1 pb-1 text-[11px] font-medium tracking-wide text-base-500 uppercase">
      🎧 {{ t("nav.recordings") }}
    </span>

    <p v-if="projects.recordings.length === 0" class="px-3 py-2 text-xs text-base-500">
      {{ t("recordings.empty") }}
    </p>

    <button
      v-for="recording in projects.recordings"
      :key="recording.id"
      type="button"
      class="flex items-center gap-2 rounded-lg px-3 py-2 text-left transition-colors"
      :class="
        projects.activeRecordingId === recording.id
          ? 'bg-base-800'
          : 'hover:bg-base-850'
      "
      @click="select(recording.id)"
    >
      <span
        class="size-2 shrink-0 rounded-full"
        :class="STATUS_COLOR[recording.status] ?? 'bg-base-500'"
      />
      <span class="min-w-0 flex-1">
        <span class="block truncate text-sm text-slate-300">{{ recording.file_name }}</span>
        <span class="block truncate text-[11px] text-base-500">
          {{ t(`status.${recording.status}`) }}
        </span>
      </span>
    </button>
  </div>
</template>
