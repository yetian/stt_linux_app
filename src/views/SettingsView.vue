<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import LanguageSelector from "@/components/LanguageSelector.vue";
import { llmApi } from "@/api/llm";
import { AUDIO_LANGUAGE_OPTIONS, useSettingsStore } from "@/stores/settings";
import type { AudioLanguage } from "@/stores/settings";
import type { LlmModel, SummaryProvider } from "@/types";

const { t } = useI18n();
const settings = useSettingsStore();

const providers: { value: SummaryProvider; label: string }[] = [
  { value: "ollama", label: "Ollama" },
  { value: "openai", label: "OpenAI-compatible (LM Studio)" },
];

interface Draft {
  provider: SummaryProvider;
  ollamaEndpoint: string;
  openaiEndpoint: string;
  ollamaModel: string;
  openaiModel: string;
}

function makeDraft(): Draft {
  return {
    provider: settings.summaryProvider,
    ollamaEndpoint: settings.ollamaEndpoint,
    openaiEndpoint: settings.openaiEndpoint,
    ollamaModel: settings.ollamaModel,
    openaiModel: settings.openaiModel,
  };
}

const draft = reactive<Draft>(makeDraft());

type Status = "idle" | "checking" | "online" | "offline";
const status = ref<Status>("idle");
const availableModels = ref<LlmModel[]>([]);
const statusError = ref<string | null>(null);
const confirmOpen = ref(false);

const draftEndpoint = computed({
  get: () =>
    draft.provider === "ollama" ? draft.ollamaEndpoint : draft.openaiEndpoint,
  set: (value: string) => {
    if (draft.provider === "ollama") draft.ollamaEndpoint = value;
    else draft.openaiEndpoint = value;
  },
});

const draftModel = computed({
  get: () => (draft.provider === "ollama" ? draft.ollamaModel : draft.openaiModel),
  set: (value: string) => {
    if (draft.provider === "ollama") draft.ollamaModel = value;
    else draft.openaiModel = value;
  },
});

const changedFields = computed(() => {
  const fields: string[] = [];
  if (draft.provider !== settings.summaryProvider) fields.push(t("settings.provider"));
  if (draft.ollamaEndpoint !== settings.ollamaEndpoint) fields.push(t("settings.ollamaEndpoint"));
  if (draft.openaiEndpoint !== settings.openaiEndpoint) fields.push(t("settings.openaiEndpoint"));
  if (draft.ollamaModel !== settings.ollamaModel) fields.push(`Ollama · ${t("settings.model")}`);
  if (draft.openaiModel !== settings.openaiModel) fields.push(`OpenAI · ${t("settings.model")}`);
  return fields;
});

const isDirty = computed(() => changedFields.value.length > 0);

const statusLabel = computed(() => {
  switch (status.value) {
    case "checking":
      return t("settings.checking");
    case "online":
      return t("settings.online");
    case "offline":
      return t("settings.offline");
    default:
      return t("settings.notChecked");
  }
});

const statusColor = computed(() => {
  switch (status.value) {
    case "online":
      return "bg-emerald-400 shadow-[0_0_8px] shadow-emerald-400/60";
    case "checking":
      return "bg-amber-400";
    case "offline":
      return "bg-rose-500";
    default:
      return "bg-base-600";
  }
});

async function testConnection(): Promise<void> {
  status.value = "checking";
  statusError.value = null;
  try {
    availableModels.value = await llmApi.listModels(draft.provider, draftEndpoint.value);
    status.value = "online";
  } catch (cause) {
    availableModels.value = [];
    status.value = "offline";
    statusError.value = String(cause);
  }
}

function applyChanges(): void {
  confirmOpen.value = false;
  settings.summaryProvider = draft.provider;
  settings.ollamaEndpoint = draft.ollamaEndpoint;
  settings.openaiEndpoint = draft.openaiEndpoint;
  settings.ollamaModel = draft.ollamaModel;
  settings.openaiModel = draft.openaiModel;
}

function discardChanges(): void {
  Object.assign(draft, makeDraft());
  void testConnection();
}

watch(
  () => draft.provider,
  () => {
    void testConnection();
  },
);

onMounted(() => {
  void testConnection();
});

function onAudioLanguage(event: Event): void {
  settings.setAudioLanguage((event.target as HTMLSelectElement).value as AudioLanguage);
}

function onProvider(event: Event): void {
  draft.provider = (event.target as HTMLSelectElement).value as SummaryProvider;
}
</script>

<template>
  <div class="mx-auto flex h-full max-w-3xl flex-col gap-6 overflow-y-auto px-6 py-6">
    <h1 class="text-lg font-semibold text-slate-100">
      <i class="fa-solid fa-gear mr-1 text-accent-400"></i>{{ t("nav.settings") }}
    </h1>

    <section class="flex flex-col gap-4 rounded-2xl border border-base-800 bg-base-900 p-5">
      <LanguageSelector />

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          <i class="fa-solid fa-headphones mr-1"></i>{{ t("language.audio") }}
        </span>
        <select
          :value="settings.audioLanguage"
          class="select-field w-full"
          @change="onAudioLanguage"
        >
          <option v-for="option in AUDIO_LANGUAGE_OPTIONS" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          <i class="fa-solid fa-file-lines mr-1"></i>{{ t("language.output") }}
        </span>
        <input v-model="settings.outputLanguage" type="text" class="field" />
      </label>
    </section>

    <section class="flex flex-col gap-4 rounded-2xl border border-base-800 bg-base-900 p-5">
      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          <i class="fa-solid fa-robot mr-1"></i>{{ t("settings.provider") }}
        </span>
        <select :value="draft.provider" class="select-field w-full" @change="onProvider">
          <option v-for="provider in providers" :key="provider.value" :value="provider.value">
            {{ provider.label }}
          </option>
        </select>
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          <i class="fa-solid fa-link mr-1"></i>
          {{ draft.provider === "ollama" ? t("settings.ollamaEndpoint") : t("settings.openaiEndpoint") }}
        </span>
        <input v-model="draftEndpoint" type="text" class="field font-mono" />
      </label>

      <div class="flex items-center justify-between gap-3 rounded-lg bg-base-850 px-3 py-2">
        <div class="flex min-w-0 items-center gap-2">
          <span class="size-2.5 shrink-0 rounded-full" :class="statusColor" />
          <span class="text-xs text-slate-300">{{ t("settings.connection") }}: {{ statusLabel }}</span>
        </div>
        <button
          type="button"
          class="shrink-0 rounded-lg border border-base-700 bg-base-800 px-3 py-1 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:opacity-40"
          :disabled="status === 'checking'"
          @click="testConnection"
        >
          <i class="fa-solid fa-plug-circle-bolt mr-1"></i>{{ t("settings.test") }}
        </button>
      </div>

      <p v-if="status === 'offline' && statusError" class="text-[11px] text-rose-400">
        {{ statusError }}
      </p>

      <label class="flex flex-col gap-1">
        <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
          <i class="fa-solid fa-puzzle-piece mr-1"></i>{{ t("settings.model") }}
        </span>
        <select
          v-if="status === 'online' && availableModels.length > 0"
          v-model="draftModel"
          class="select-field w-full"
        >
          <option value="">{{ t("settings.model") }}</option>
          <option
            v-if="draftModel && !availableModels.some((model) => model.name === draftModel)"
            :value="draftModel"
          >
            {{ draftModel }}
          </option>
          <option v-for="model in availableModels" :key="model.name" :value="model.name">
            {{ model.name }}
          </option>
        </select>
        <input v-else v-model="draftModel" type="text" class="field font-mono" />
        <span v-if="status === 'online' && availableModels.length > 0" class="text-[10px] text-base-500">
          {{ t("settings.fromServer") }}
        </span>
      </label>

      <div
        v-if="isDirty"
        class="flex items-center justify-between gap-3 rounded-lg border border-accent-500/40 bg-accent-500/5 px-3 py-2"
      >
        <span class="text-xs text-accent-300">
          <i class="fa-solid fa-circle-exclamation mr-1"></i>{{ t("settings.unsaved") }}
        </span>
        <span class="flex items-center gap-2">
          <button
            type="button"
            class="rounded-lg border border-base-700 bg-base-850 px-3 py-1 text-xs font-medium text-slate-300 hover:text-slate-100"
            @click="discardChanges"
          >
            {{ t("settings.discard") }}
          </button>
          <button
            type="button"
            class="rounded-lg bg-accent-500 px-3 py-1 text-xs font-medium text-base-950 hover:bg-accent-400"
            @click="confirmOpen = true"
          >
            {{ t("settings.apply") }}
          </button>
        </span>
      </div>
    </section>

    <section class="flex flex-col gap-3 rounded-2xl border border-base-800 bg-base-900 p-5">
      <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
        <i class="fa-solid fa-microchip mr-1"></i>{{ t("settings.performance") }}
      </span>
      <label class="flex items-center justify-between gap-3">
        <span class="text-xs text-slate-300">
          {{ t("settings.useGpu") }}
          <span class="mt-0.5 block text-[10px] text-base-500">{{ t("settings.useGpuHint") }}</span>
        </span>
        <input v-model="settings.useGpu" type="checkbox" class="size-4 accent-sky-500" />
      </label>
    </section>

    <ConfirmDialog
      :open="confirmOpen"
      :title="t('settings.confirmTitle')"
      :message="t('settings.confirmMessage', { fields: changedFields.join(', ') })"
      :confirm-label="t('settings.apply')"
      @confirm="applyChanges"
      @cancel="confirmOpen = false"
    />
  </div>
</template>
