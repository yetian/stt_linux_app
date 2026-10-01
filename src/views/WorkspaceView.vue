<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import Dropzone from "@/components/Dropzone.vue";
import ProcessingTimeline from "@/components/ProcessingTimeline.vue";
import SummaryPanel from "@/components/SummaryPanel.vue";
import TranscriptPanel from "@/components/TranscriptPanel.vue";
import { diarizationApi } from "@/api/diarization";
import { llmApi } from "@/api/llm";
import { projectApi } from "@/api/projects";
import { sttApi } from "@/api/stt";
import { useModelsStore } from "@/stores/models";
import { usePipelineStore } from "@/stores/pipeline";
import { useProjectsStore } from "@/stores/projects";
import { useSettingsStore } from "@/stores/settings";
import type { DiarizedSegment, TranscriptSegment } from "@/types";

const { t } = useI18n();
const pipeline = usePipelineStore();
const projects = useProjectsStore();
const settings = useSettingsStore();
const models = useModelsStore();

const TAB_ICONS = { summary: "📄", transcript: "💬" } as const;

const activeTab = ref<"summary" | "transcript">("summary");
const segments = ref<TranscriptSegment[]>([]);
const diarized = ref<DiarizedSegment[]>([]);
const exporting = ref(false);

const recording = computed(() => projects.activeRecording);
const canTranscribe = computed(() => models.downloadedStt !== null);
const canDiarize = computed(
  () => models.downloadedDiarization !== null && segments.value.length > 0,
);

async function runTranscribe(): Promise<void> {
  const current = recording.value;
  const model = models.downloadedStt;
  if (!current || !model) return;

  pipeline.currentFile = current.file_name;
  pipeline.setProgress(10);
  pipeline.setStage("transcribing");

  try {
    segments.value = await sttApi.transcribe(
      current.id,
      model.path,
      settings.audioLanguage,
    );
    diarized.value = [];
    pipeline.setProgress(45);
    await projects.loadRecordings();
    pipeline.setStage("idle");
  } catch (cause) {
    pipeline.fail(String(cause));
  }
}

async function runDiarize(): Promise<void> {
  const current = recording.value;
  const model = models.downloadedDiarization;
  if (!current || !model || segments.value.length === 0) return;

  pipeline.setStage("clustering");
  pipeline.setProgress(65);

  try {
    diarized.value = await diarizationApi.diarize(
      current.id,
      model.path,
      segments.value,
    );
    pipeline.setProgress(80);
    await projects.loadRecordings();
    pipeline.setStage("idle");
  } catch (cause) {
    pipeline.fail(String(cause));
  }
}

async function runSummarize(): Promise<void> {
  const current = recording.value;
  if (!current) return;

  pipeline.setStage("summarizing");
  pipeline.setProgress(88);

  try {
    await llmApi.summarizeRecording(current.id, {
      provider: settings.summaryProvider,
      endpoint:
        settings.summaryProvider === "ollama"
          ? settings.ollamaEndpoint
          : settings.openaiEndpoint,
      model: settings.ollamaModel,
      targetLanguage: settings.outputLanguage,
    });
    pipeline.setProgress(100);
    await projects.loadRecordings();
    pipeline.setStage("idle");
  } catch (cause) {
    pipeline.fail(String(cause));
  }
}

async function runExport(): Promise<void> {
  const current = recording.value;
  if (!current) return;
  exporting.value = true;
  try {
    await projectApi.exportRecording(current.id);
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <div class="mx-auto flex max-w-5xl flex-col gap-5 px-6 py-6">
    <Dropzone />

    <ProcessingTimeline />

    <section v-if="!recording" class="grid place-items-center rounded-2xl border border-base-800 bg-base-900 px-6 py-16 text-center">
      <div class="max-w-sm">
        <p class="text-sm font-medium text-slate-300">🎙️ {{ t("workspace.selectRecording") }}</p>
        <p class="mt-1 text-xs text-base-500">{{ t("workspace.selectRecordingHint") }}</p>
      </div>
    </section>

    <template v-else>
      <div class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-base-800 bg-base-900 px-5 py-4">
        <div class="min-w-0">
          <p class="truncate text-sm font-semibold text-slate-100">{{ recording.file_name }}</p>
          <p class="truncate font-mono text-[11px] text-base-500">{{ recording.source_path }}</p>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <button
            type="button"
            class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="!canTranscribe || pipeline.isRunning"
            @click="runTranscribe"
          >
            📝 {{ t("actions.transcribe") }}
          </button>
          <button
            type="button"
            class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="!canDiarize || pipeline.isRunning"
            @click="runDiarize"
          >
            👥 {{ t("actions.diarize") }}
          </button>
          <button
            type="button"
            class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="pipeline.isRunning"
            @click="runSummarize"
          >
            ✨ {{ t("actions.summarize") }}
          </button>
          <button
            type="button"
            class="rounded-lg bg-accent-500 px-3 py-1.5 text-xs font-medium text-base-950 transition-colors hover:bg-accent-400 disabled:opacity-40"
            :disabled="exporting"
            @click="runExport"
          >
            💾 {{ t("actions.export") }}
          </button>
        </div>
      </div>

      <section class="rounded-2xl border border-base-800 bg-base-900">
        <div class="flex items-center gap-1 border-b border-base-800 p-2">
          <button
            v-for="tab in (['summary', 'transcript'] as const)"
            :key="tab"
            type="button"
            class="rounded-lg px-4 py-2 text-sm font-medium transition-colors"
            :class="
              activeTab === tab
                ? 'bg-base-800 text-slate-100'
                : 'text-base-500 hover:text-slate-300'
            "
            @click="activeTab = tab"
          >
            {{ TAB_ICONS[tab] }} {{ t(`tabs.${tab}`) }}
          </button>
        </div>

        <div class="p-4">
          <SummaryPanel
            v-if="activeTab === 'summary'"
            :markdown="recording.summary_markdown"
          />
          <TranscriptPanel
            v-else
            :transcript="recording.transcript_raw"
            :diarized="diarized"
          />
        </div>
      </section>
    </template>
  </div>
</template>
