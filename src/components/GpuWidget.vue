<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const gpuName = "RTX 4060";
const totalVramGb = 8;
const usedVramGb = 0.5;

const percent = computed(() =>
  Math.min(100, Math.round((usedVramGb / totalVramGb) * 100)),
);
</script>

<template>
  <div class="flex min-w-52 flex-col gap-1.5">
    <div class="flex items-center justify-between text-[11px]">
      <span class="font-medium tracking-wide text-base-500 uppercase">
        <i class="fa-solid fa-microchip mr-1"></i>{{ t("header.gpu") }} · {{ gpuName }}
      </span>
      <span class="font-mono text-slate-300">
        {{ usedVramGb.toFixed(1) }} / {{ totalVramGb }} GB
      </span>
    </div>
    <div class="h-1.5 w-full overflow-hidden rounded-full bg-base-800">
      <div
        class="h-full rounded-full bg-gradient-to-r from-accent-600 to-accent-400 transition-all"
        :style="{ width: `${percent}%` }"
      />
    </div>
  </div>
</template>
