<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { systemApi } from "@/api/system";

const { t } = useI18n();

const gpuName = ref("—");
const vramLabel = ref("—");

function formatVram(mb: number): string {
  const gb = mb / 1024;
  return `${Number.isInteger(gb) ? gb : gb.toFixed(1)} GB`;
}

onMounted(async () => {
  try {
    const info = await systemApi.getGpuInfo();
    if (!info) return;
    gpuName.value = info.name;
    vramLabel.value = formatVram(info.vram_mb);
  } catch {
    /* keep placeholders */
  }
});
</script>

<template>
  <div :aria-label="t('header.gpu')" class="flex items-center gap-2.5">
    <i class="fa-solid fa-microchip shrink-0 text-base-500" aria-hidden="true"></i>
    <div class="flex min-w-0 flex-col leading-tight">
      <span class="truncate text-xs font-medium text-slate-200">{{ gpuName }}</span>
      <span class="font-mono text-[11px] text-base-500">
        {{ t("header.vram") }} · {{ vramLabel }}
      </span>
    </div>
  </div>
</template>
