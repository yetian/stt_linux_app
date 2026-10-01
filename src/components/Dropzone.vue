<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { useProjectsStore } from "@/stores/projects";

const { t } = useI18n();
const projects = useProjectsStore();

const picking = ref(false);

async function browse(): Promise<void> {
  if (picking.value) return;
  picking.value = true;
  try {
    const selection = await open({
      multiple: true,
      directory: false,
      filters: [
        { name: "Audio", extensions: ["mp3", "wav", "m4a", "aac", "flac"] },
      ],
    });

    if (!selection) return;
    const paths = Array.isArray(selection) ? selection : [selection];
    await projects.addRecordingsFromPaths(paths);
  } finally {
    picking.value = false;
  }
}
</script>

<template>
  <div
    class="flex cursor-pointer items-center gap-3 rounded-xl border border-dashed border-base-700 bg-base-900 px-4 py-3 transition-colors hover:border-accent-500 hover:bg-base-850"
    @click="browse"
  >
    <div class="shrink-0 rounded-lg bg-base-800 p-2 text-accent-400">
      <i class="fa-solid fa-file-audio"></i>
    </div>
    <div class="min-w-0 flex-1">
      <p class="truncate text-sm font-medium text-slate-200">{{ t("dropzone.title") }}</p>
      <p class="truncate text-xs text-base-500">{{ t("dropzone.subtitle") }}</p>
    </div>
    <span
      class="shrink-0 rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300"
    >
      <i class="fa-solid fa-folder-open mr-1"></i>{{ t("dropzone.browse") }}
    </span>
  </div>
</template>
