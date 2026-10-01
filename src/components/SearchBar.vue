<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const query = defineModel<string>({ default: "" });
</script>

<template>
  <div class="relative">
    <span class="pointer-events-none absolute inset-y-0 left-3 flex items-center text-sm text-base-500">
      🔍
    </span>
    <input
      v-model="query"
      type="search"
      :placeholder="t('search.placeholder')"
      class="w-full rounded-lg border border-base-700 bg-base-850 py-2 pr-3 pl-9 text-sm text-slate-200 outline-none transition-colors placeholder:text-base-500 hover:border-base-600 focus:border-accent-500"
    />
  </div>
</template>
