<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { systemApi } from "@/api/system";
import { useModelsStore } from "@/stores/models";
import type { ModelStatus } from "@/types";

const { t } = useI18n();
const models = useModelsStore();

const directory = ref("");

onMounted(async () => {
  await models.initialize();
  try {
    directory.value = (await systemApi.getPaths()).models_dir;
  } catch {
    directory.value = "";
  }
});

const sections = computed(() => [
  { key: "stt", label: `🗣️ ${t("models.speechModels")}`, items: models.sttModels },
  {
    key: "diarization",
    label: `👥 ${t("models.speakerModels")}`,
    items: models.diarizationModels,
  },
]);

function percent(model: ModelStatus): number {
  return models.downloadPercent(model.filename);
}
</script>

<template>
  <div class="mx-auto flex max-w-4xl flex-col gap-6 px-6 py-6">
    <header>
      <h1 class="text-lg font-semibold text-slate-100">🧠 {{ t("models.title") }}</h1>
      <p class="mt-1 text-xs text-base-500">{{ t("models.subtitle") }}</p>
      <p v-if="directory" class="mt-2 font-mono text-[11px] text-base-600">
        {{ t("models.directory") }}: {{ directory }}
      </p>
    </header>

    <section
      v-for="section in sections"
      :key="section.key"
      class="flex flex-col gap-3"
    >
      <h2 class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
        {{ section.label }}
      </h2>

      <article
        v-for="model in section.items"
        :key="model.id"
        class="rounded-2xl border border-base-800 bg-base-900 p-5"
      >
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="min-w-0">
            <p class="text-sm font-medium text-slate-200">{{ model.name }}</p>
            <p class="mt-0.5 text-xs text-base-500">
              {{ model.size_mb }} MB ·
              {{ model.downloaded ? t("models.downloaded") : t("models.notDownloaded") }}
            </p>
          </div>

          <div class="flex items-center gap-2">
            <button
              v-if="!model.downloaded"
              type="button"
              class="rounded-lg bg-accent-500 px-3 py-1.5 text-xs font-medium text-base-950 transition-colors hover:bg-accent-400 disabled:opacity-40"
              :disabled="models.isDownloading(model.filename)"
              @click="models.download(model)"
            >
              ⬇️ {{ t("models.download") }}
            </button>
            <button
              v-else
              type="button"
              class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-rose-500 hover:text-rose-400"
              @click="models.remove(model)"
            >
              🗑️ {{ t("models.delete") }}
            </button>
          </div>
        </div>

        <div
          v-if="models.isDownloading(model.filename)"
          class="mt-3 h-1.5 w-full overflow-hidden rounded-full bg-base-800"
        >
          <div
            class="h-full rounded-full bg-gradient-to-r from-accent-600 to-accent-400 transition-all"
            :style="{ width: `${percent(model)}%` }"
          />
        </div>
      </article>
    </section>

    <p v-if="models.error" class="text-xs text-rose-400">{{ models.error }}</p>
  </div>
</template>
