<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{ markdown?: string | null }>();

const { t } = useI18n();
</script>

<template>
  <div
    v-if="markdown"
    class="prose-invert max-h-[60vh] overflow-y-auto rounded-xl bg-base-850 p-5 text-sm leading-relaxed whitespace-pre-wrap text-slate-300"
  >
    {{ markdown }}
  </div>

  <div v-else class="grid place-items-center rounded-2xl border border-base-800 bg-base-900 px-6 py-16 text-center">
    <div class="max-w-sm">
      <div class="mx-auto grid size-12 place-items-center rounded-full bg-base-800 text-base-500">
        <svg viewBox="0 0 24 24" fill="none" class="size-6" stroke="currentColor" stroke-width="1.6">
          <path d="M5 4h14v16H5z" stroke-linejoin="round" />
          <path d="M9 8h6M9 12h6M9 16h3" stroke-linecap="round" />
        </svg>
      </div>
      <p class="mt-4 text-sm font-medium text-slate-300">{{ t("summary.empty") }}</p>
      <p class="mt-1 text-xs text-base-500">{{ t("summary.emptyHint") }}</p>
    </div>
  </div>
</template>
