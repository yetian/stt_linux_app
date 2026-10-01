<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { isUiLocale, type UiLocale } from "@/i18n";
import { useSettingsStore } from "@/stores/settings";

const { t } = useI18n();
const settings = useSettingsStore();

const options: { value: UiLocale; label: string }[] = [
  { value: "en", label: "English" },
  { value: "zh-CN", label: "简体中文" },
  { value: "de", label: "Deutsch" },
];

const selected = computed({
  get: () => settings.uiLocale as string,
  set: (value: string) => {
    if (isUiLocale(value)) {
      settings.setLocale(value);
    }
  },
});
</script>

<template>
  <label class="flex flex-col gap-1">
    <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
      <i class="fa-solid fa-globe mr-1"></i>{{ t("language.ui") }}
    </span>
    <select v-model="selected" class="select-field w-full">
      <option v-for="option in options" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </select>
  </label>
</template>
