<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>

<template>
  <div
    class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-base-700 bg-base-900 px-8 py-10 text-center"
  >
    <div class="rounded-full bg-base-800 p-4 text-accent-400">
      <i class="fa-solid fa-file-audio text-xl"></i>
    </div>
    <div>
      <p class="text-sm font-medium text-slate-200">{{ t("dropzone.title") }}</p>
      <p class="mt-1 text-xs text-base-500">{{ t("dropzone.subtitle") }}</p>
    </div>
  </div>
</template>
