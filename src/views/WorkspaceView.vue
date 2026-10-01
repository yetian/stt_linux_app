<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import Dropzone from "@/components/Dropzone.vue";
import ProcessingTimeline from "@/components/ProcessingTimeline.vue";
import SummaryPanel from "@/components/SummaryPanel.vue";
import TextInputDialog from "@/components/TextInputDialog.vue";
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

const TAB_ICONS = { summary: "fa-file-lines", transcript: "fa-comments" } as const;

const activeTab = ref<"summary" | "transcript">("transcript");
const segments = ref<TranscriptSegment[]>([]);
const diarized = ref<DiarizedSegment[]>([]);
const exporting = ref(false);
const renameOpen = ref(false);
const deleteOpen = ref(false);
const newTag = ref("");

const recording = computed(() => projects.activeRecording);
const tags = computed(() =>
  recording.value ? projects.recordingTags(recording.value.id) : [],
);
const canTranscribe = computed(() => models.downloadedStt !== null);
const canDiarize = computed(
  () => models.downloadedDiarization !== null && segments.value.length > 0,
);

watch(
  () => recording.value?.id ?? null,
  (id) => {
    if (id) void projects.loadTags(id);
  },
  { immediate: true },
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

async function submitRename(name: string): Promise<void> {
  const current = recording.value;
  renameOpen.value = false;
  if (current) await projects.renameRecording(current.id, name);
}

async function confirmDelete(): Promise<void> {
  const current = recording.value;
  deleteOpen.value = false;
  if (current) await projects.removeRecording(current.id);
}

async function onMoveProject(event: Event): Promise<void> {
  const current = recording.value;
  if (!current) return;
  const value = (event.target as HTMLSelectElement).value;
  await projects.assignRecording(current.id, value === "" ? null : value);
}

async function submitTag(): Promise<void> {
  const current = recording.value;
  const name = newTag.value.trim();
  if (!current || name === "") return;
  newTag.value = "";
  await projects.addTag(current.id, name);
}

async function removeTag(tagId: number): Promise<void> {
  const current = recording.value;
  if (current) await projects.removeTag(current.id, tagId);
}
</script>

<template>
  <div class="mx-auto flex h-full max-w-5xl flex-col gap-5 px-6 py-6">
    <Dropzone />

    <ProcessingTimeline />

    <section
      v-if="!recording"
      class="grid place-items-center rounded-2xl border border-base-800 bg-base-900 px-6 py-16 text-center"
    >
      <div class="max-w-sm">
        <p class="text-sm font-medium text-slate-300">
          <i class="fa-solid fa-microphone mr-1"></i>{{ t("workspace.selectRecording") }}
        </p>
        <p class="mt-1 text-xs text-base-500">{{ t("workspace.selectRecordingHint") }}</p>
      </div>
    </section>

    <template v-else>
      <div class="flex flex-col gap-3 rounded-2xl border border-base-800 bg-base-900 px-5 py-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex min-w-0 items-center gap-2">
            <p class="truncate text-sm font-semibold text-slate-100">
              <i class="fa-solid fa-music mr-1 text-base-500"></i>{{ recording.file_name }}
            </p>
            <button
              type="button"
              class="shrink-0 text-base-500 transition-colors hover:text-accent-400"
              :title="t('recordings.renameTitle')"
              @click="renameOpen = true"
            >
              <i class="fa-solid fa-pen"></i>
            </button>
            <button
              type="button"
              class="shrink-0 text-base-500 transition-colors hover:text-rose-400"
              :title="t('recordings.deleteTitle')"
              @click="deleteOpen = true"
            >
              <i class="fa-solid fa-trash"></i>
            </button>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <button
              type="button"
              class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:cursor-not-allowed disabled:opacity-40"
              :disabled="!canTranscribe || pipeline.isRunning"
              @click="runTranscribe"
            >
              <i class="fa-solid fa-file-pen mr-1"></i>{{ t("actions.transcribe") }}
            </button>
            <button
              type="button"
              class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:cursor-not-allowed disabled:opacity-40"
              :disabled="!canDiarize || pipeline.isRunning"
              @click="runDiarize"
            >
              <i class="fa-solid fa-users mr-1"></i>{{ t("actions.diarize") }}
            </button>
            <button
              type="button"
              class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:cursor-not-allowed disabled:opacity-40"
              :disabled="pipeline.isRunning"
              @click="runSummarize"
            >
              <i class="fa-solid fa-wand-magic-sparkles mr-1"></i>{{ t("actions.summarize") }}
            </button>
            <button
              type="button"
              class="rounded-lg bg-accent-500 px-3 py-1.5 text-xs font-medium text-base-950 transition-colors hover:bg-accent-400 disabled:opacity-40"
              :disabled="exporting"
              @click="runExport"
            >
              <i class="fa-solid fa-floppy-disk mr-1"></i>{{ t("actions.export") }}
            </button>
          </div>
        </div>

        <p class="truncate font-mono text-[11px] text-base-500">{{ recording.source_path }}</p>

        <div class="flex flex-wrap items-center gap-4">
          <label class="flex items-center gap-2 text-xs text-base-500">
            <span><i class="fa-solid fa-folder mr-1"></i>{{ t("recordings.moveTo") }}</span>
            <select
              :value="recording.project_id ?? ''"
              class="rounded-lg border border-base-700 bg-base-850 px-2 py-1 text-xs text-slate-200 outline-none focus:border-accent-500"
              @change="onMoveProject"
            >
              <option value="">{{ t("recordings.noProject") }}</option>
              <option v-for="project in projects.projects" :key="project.id" :value="project.id">
                {{ project.name }}
              </option>
            </select>
          </label>

          <div class="flex flex-wrap items-center gap-2">
            <span class="text-xs text-base-500"><i class="fa-solid fa-tags"></i></span>
            <span
              v-for="tag in tags"
              :key="tag.id"
              class="inline-flex items-center gap-1 rounded-full bg-base-800 px-2 py-0.5 text-[11px] text-slate-300"
            >
              {{ tag.tag_name }}
              <button
                type="button"
                class="text-base-500 transition-colors hover:text-rose-400"
                @click="removeTag(tag.id)"
              >
                <i class="fa-solid fa-xmark"></i>
              </button>
            </span>
            <input
              v-model="newTag"
              type="text"
              :placeholder="t('recordings.tagPlaceholder')"
              class="w-24 rounded-md border border-base-700 bg-base-850 px-2 py-0.5 text-[11px] text-slate-200 outline-none placeholder:text-base-500 focus:border-accent-500"
              @keydown.enter.prevent="submitTag"
            />
            <button
              type="button"
              class="text-base-500 transition-colors hover:text-accent-400"
              :title="t('recordings.addTag')"
              @click="submitTag"
            >
              <i class="fa-solid fa-plus"></i>
            </button>
          </div>
        </div>
      </div>

      <section class="flex min-h-0 flex-1 flex-col rounded-2xl border border-base-800 bg-base-900">
        <div class="flex shrink-0 items-center gap-1 border-b border-base-800 p-2">
          <button
            v-for="tab in (['transcript', 'summary'] as const)"
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
            <i :class="['fa-solid', TAB_ICONS[tab], 'mr-1']"></i>{{ t(`tabs.${tab}`) }}
          </button>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-4">
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

    <TextInputDialog
      :open="renameOpen"
      :title="t('recordings.renameTitle')"
      :label="t('recordings.renameLabel')"
      :initial-value="recording?.file_name"
      @confirm="submitRename"
      @cancel="renameOpen = false"
    />

    <ConfirmDialog
      :open="deleteOpen"
      :title="t('recordings.deleteTitle')"
      :message="t('recordings.deleteMessage', { name: recording?.file_name })"
      :confirm-label="t('common.delete')"
      danger
      @confirm="confirmDelete"
      @cancel="deleteOpen = false"
    />
  </div>
</template>
