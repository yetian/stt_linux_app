<script setup lang="ts">
import { useI18n } from "vue-i18n";
import SidebarSection from "@/components/SidebarSection.vue";
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
    recorded_at: file.modified_at,
  });
  await projects.loadProjects();
}

function added(path: string): boolean {
  return projects.hasRecordingWithPath(path);
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
  <SidebarSection v-if="device.files.length > 0" :title="t('nav.deviceFiles')" icon="fa-usb">
    <div
      v-for="file in device.files"
      :key="file.path"
      class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5"
    >
      <div class="min-w-0 flex-1">
        <p class="truncate text-xs text-slate-300" :title="file.path">
          <i class="fa-solid fa-music mr-1 text-base-500"></i>{{ file.name }}
        </p>
        <p class="truncate text-[10px] text-base-500">{{ metadata(file) }}</p>
      </div>
      <button
        type="button"
        class="shrink-0 rounded-md p-1 transition-colors"
        :class="
          added(file.path)
            ? 'cursor-default text-emerald-400'
            : 'text-accent-400 hover:bg-base-800 hover:text-accent-300'
        "
        :title="added(file.path) ? t('recordings.added') : t('recordings.add')"
        :disabled="added(file.path)"
        @click.stop="add(file)"
      >
        <i :class="added(file.path) ? 'fa-solid fa-circle-check' : 'fa-solid fa-plus'"></i>
      </button>
    </div>
  </SidebarSection>
</template>
