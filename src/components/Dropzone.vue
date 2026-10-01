<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>

<template>
  <div
    class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-base-700 bg-base-900 px-8 py-10 text-center"
  >
    <div class="rounded-full bg-base-800 p-4 text-accent-400">
      <svg viewBox="0 0 24 24" fill="none" class="size-7" stroke="currentColor" stroke-width="1.8">
        <path d="M12 16V4m0 0L8 8m4-4 4 4" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" stroke-linecap="round" />
      </svg>
    </div>
    <div>
      <p class="text-sm font-medium text-slate-200">{{ t("dropzone.title") }}</p>
      <p class="mt-1 text-xs text-base-500">{{ t("dropzone.subtitle") }}</p>
    </div>
  </div>
</template>
