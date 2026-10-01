<script setup lang="ts">
import { useI18n } from "vue-i18n";
import LanguageSelector from "@/components/LanguageSelector.vue";
import { AUDIO_LANGUAGE_OPTIONS, useSettingsStore } from "@/stores/settings";
import type { AudioLanguage } from "@/stores/settings";
import type { SummaryProvider } from "@/types";

const { t } = useI18n();
const settings = useSettingsStore();

const providers: { value: SummaryProvider; label: string }[] = [
  { value: "ollama", label: "Ollama" },
  { value: "openai", label: "OpenAI-compatible (LM Studio)" },
];

function onAudioLanguage(event: Event): void {
  settings.setAudioLanguage((event.target as HTMLSelectElement).value as AudioLanguage);
}

function onProvider(event: Event): void {
  settings.summaryProvider = (event.target as HTMLSelectElement).value as SummaryProvider;
}
</script>

<template>
  <div class="mx-auto flex max-w-3xl flex-col gap-6 px-6 py-6">
    <h1 class="text-lg font-semibold text-slate-100">{{ t("nav.settings") }}</h1>

    <section class="flex flex-col gap-4 rounded-2xl border border-base-800 bg-base-900 p-5">
      <LanguageSelector />

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          {{ t("language.audio") }}
        </span>
        <select
          :value="settings.audioLanguage"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-2 text-sm text-slate-200 outline-none focus:border-accent-500"
          @change="onAudioLanguage"
        >
          <option v-for="option in AUDIO_LANGUAGE_OPTIONS" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          {{ t("language.output") }}
        </span>
        <input
          v-model="settings.outputLanguage"
          type="text"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-2 text-sm text-slate-200 outline-none focus:border-accent-500"
        />
      </label>
    </section>

    <section class="flex flex-col gap-4 rounded-2xl border border-base-800 bg-base-900 p-5">
      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          {{ t("settings.provider") }}
        </span>
        <select
          :value="settings.summaryProvider"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-2 text-sm text-slate-200 outline-none focus:border-accent-500"
          @change="onProvider"
        >
          <option v-for="provider in providers" :key="provider.value" :value="provider.value">
            {{ provider.label }}
          </option>
        </select>
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          {{ t("settings.ollamaEndpoint") }}
        </span>
        <input
          v-model="settings.ollamaEndpoint"
          type="text"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-2 font-mono text-sm text-slate-200 outline-none focus:border-accent-500"
        />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          {{ t("settings.openaiEndpoint") }}
        </span>
        <input
          v-model="settings.openaiEndpoint"
          type="text"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-2 font-mono text-sm text-slate-200 outline-none focus:border-accent-500"
        />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          {{ t("settings.model") }}
        </span>
        <input
          v-model="settings.ollamaModel"
          type="text"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-2 font-mono text-sm text-slate-200 outline-none focus:border-accent-500"
        />
      </label>
    </section>
  </div>
</template>
