<script setup lang="ts">
import { useI18n } from "vue-i18n";
import DeviceStatus from "@/components/DeviceStatus.vue";
import GpuWidget from "@/components/GpuWidget.vue";
import { useProjectsStore } from "@/stores/projects";

const { t } = useI18n();
const projects = useProjectsStore();
</script>

<template>
  <header
    class="flex flex-wrap items-center justify-between gap-4 border-b border-base-800 bg-base-900/80 px-6 py-3 backdrop-blur"
  >
    <div class="min-w-0">
      <h1 class="truncate text-base font-semibold text-slate-100">
        {{ projects.activeProject?.name ?? t("nav.workspace") }}
      </h1>
      <p class="truncate text-[11px] text-base-500">{{ t("app.tagline") }}</p>
    </div>

    <div class="flex items-center gap-6">
      <DeviceStatus />
      <GpuWidget class="hidden lg:flex" />
    </div>
  </header>
</template>
