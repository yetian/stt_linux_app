<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useDeviceStore } from "@/stores/device";

const { t } = useI18n();
const device = useDeviceStore();

const label = computed(() =>
  device.isConnected ? t("header.deviceConnected") : t("header.noDevice"),
);

const detail = computed(() => device.mountPath ?? "—");
</script>

<template>
  <div class="flex items-center gap-2.5">
    <span
      class="size-2.5 shrink-0 rounded-full"
      :class="device.isConnected ? 'bg-emerald-400 shadow-[0_0_10px] shadow-emerald-400/60' : 'bg-base-600'"
    />
    <div class="flex min-w-0 flex-col leading-tight">
      <span class="text-xs font-medium text-slate-200">{{ label }}</span>
      <span class="truncate font-mono text-[11px] text-base-500">{{ detail }}</span>
    </div>
  </div>
</template>
