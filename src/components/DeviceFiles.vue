<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useDeviceStore } from "@/stores/device";
import { useProjectsStore } from "@/stores/projects";
import type { RecorderFile } from "@/types";

const { t } = useI18n();
const device = useDeviceStore();
const projects = useProjectsStore();

async function add(file: RecorderFile): Promise<void> {
  await projects.addRecording({
    file_name: file.name,
    source_path: file.path,
    project_id: projects.activeProjectId,
    audio_duration_secs: file.duration_secs,
  });
  await projects.loadProjects();
}

function formatDuration(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds)) return "";
  const total = Math.round(seconds);
  const minutes = Math.floor(total / 60);
  const secs = total % 60;
  return `${minutes}:${String(secs).padStart(2, "0")}`;
}

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${bytes} B`;
}

function formatDate(seconds: number | null): string {
  if (seconds === null) return "";
  return new Date(seconds * 1000).toLocaleDateString();
}

function metadata(file: RecorderFile): string {
  return [
    formatDate(file.modified_at),
    formatDuration(file.duration_secs),
    formatSize(file.size_bytes),
  ]
    .filter((part) => part !== "")
    .join(" · ");
}
</script>

<template>
  <div v-if="device.files.length > 0" class="flex flex-col gap-1">
    <span class="px-1 pb-1 text-[11px] font-medium tracking-wide text-base-500 uppercase">
      <i class="fa-solid fa-usb mr-1"></i>{{ t("nav.deviceFiles") }}
    </span>

    <div class="max-h-52 overflow-y-auto pr-1">
      <div
        v-for="file in device.files"
        :key="file.path"
        class="flex items-center gap-2 rounded-lg px-2.5 py-1.5"
      >
        <div class="min-w-0 flex-1">
          <p class="truncate text-xs text-slate-300">
            <i class="fa-solid fa-music mr-1 text-base-500"></i>{{ file.name }}
          </p>
          <p class="truncate text-[10px] text-base-500">{{ metadata(file) }}</p>
          <p class="truncate font-mono text-[10px] text-base-600" :title="file.path">
            {{ file.path }}
          </p>
        </div>
        <button
          type="button"
          class="shrink-0 rounded-md p-1 text-accent-400 transition-colors hover:bg-base-800 hover:text-accent-300"
          :title="t('recordings.add')"
          @click.stop="add(file)"
        >
          <i class="fa-solid fa-plus"></i>
        </button>
      </div>
    </div>
  </div>
</template>
