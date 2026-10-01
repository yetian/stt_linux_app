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
    audio_duration_secs: null,
  });
  await projects.loadProjects();
}
</script>

<template>
  <div v-if="device.files.length > 0" class="flex flex-col gap-1">
    <span class="px-1 pb-1 text-[11px] font-medium tracking-wide text-base-500 uppercase">
      <i class="fa-solid fa-usb mr-1"></i>{{ t("nav.deviceFiles") }}
    </span>

    <button
      v-for="file in device.files"
      :key="file.path"
      type="button"
      class="flex items-center justify-between gap-2 rounded-lg px-3 py-2 text-left transition-colors hover:bg-base-850"
      @click="add(file)"
    >
      <span class="min-w-0 flex-1 truncate text-sm text-slate-300">
        <i class="fa-solid fa-music mr-1 text-base-500"></i>{{ file.name }}
      </span>
      <span class="text-[11px] text-accent-400"><i class="fa-solid fa-plus"></i></span>
    </button>
  </div>
</template>
