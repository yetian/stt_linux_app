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
    class="flex cursor-pointer flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-base-700 bg-base-900 px-8 py-10 text-center transition-colors hover:border-accent-500 hover:bg-base-850"
    @click="browse"
  >
    <div class="rounded-full bg-base-800 p-4 text-accent-400">
      <i class="fa-solid fa-file-audio text-xl"></i>
    </div>
    <div>
      <p class="text-sm font-medium text-slate-200">{{ t("dropzone.title") }}</p>
      <p class="mt-1 text-xs text-base-500">{{ t("dropzone.subtitle") }}</p>
    </div>
    <span
      class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300"
    >
      <i class="fa-solid fa-folder-open mr-1"></i>{{ t("dropzone.browse") }}
    </span>
  </div>
</template>
