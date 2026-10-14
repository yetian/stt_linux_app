import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { groupApi } from "@/api/groups";
import { llmApi } from "@/api/llm";
import { sttApi } from "@/api/stt";
import { useModelsStore } from "@/stores/models";
import { usePipelineStore } from "@/stores/pipeline";
import { useSettingsStore } from "@/stores/settings";
import type { Group, Recording } from "@/types";

export const useGroupsStore = defineStore("groups", () => {
  const groups = ref<Group[]>([]);
  const recordings = ref<Recording[]>([]);
  const activeGroupId = ref<string | null>(null);
  const loading = ref(false);
  const processing = ref(false);
  const processLabel = ref<string | null>(null);
  const error = ref<string | null>(null);

  const activeGroup = computed(
    () => groups.value.find((group) => group.id === activeGroupId.value) ?? null,
  );

  const summary = computed(() => activeGroup.value?.summary_markdown ?? null);

  async function loadGroups(): Promise<void> {
    loading.value = true;
    try {
      groups.value = await groupApi.list();
      if (
        activeGroupId.value &&
        !groups.value.some((group) => group.id === activeGroupId.value)
      ) {
        activeGroupId.value = null;
        recordings.value = [];
      }
    } catch (cause) {
      error.value = String(cause);
    } finally {
      loading.value = false;
    }
  }

  async function loadRecordings(): Promise<void> {
    if (!activeGroupId.value) {
      recordings.value = [];
      return;
    }
    try {
      recordings.value = await groupApi.listRecordings(activeGroupId.value);
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function selectGroup(groupId: string | null): Promise<void> {
    activeGroupId.value = groupId;
    await loadRecordings();
  }

  async function createGroup(name: string): Promise<Group | null> {
    error.value = null;
    try {
      const group = await groupApi.create(name);
      await loadGroups();
      await selectGroup(group.id);
      return group;
    } catch (cause) {
      error.value = String(cause);
      return null;
    }
  }

  async function renameGroup(groupId: string, name: string): Promise<void> {
    error.value = null;
    try {
      await groupApi.rename(groupId, name);
      await loadGroups();
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function removeGroup(groupId: string): Promise<void> {
    error.value = null;
    try {
      await groupApi.remove(groupId);
      if (activeGroupId.value === groupId) {
        activeGroupId.value = null;
        recordings.value = [];
      }
      await loadGroups();
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function assign(recordingId: string, groupId: string | null): Promise<void> {
    error.value = null;
    try {
      await groupApi.assign(recordingId, groupId);
      await loadGroups();
      await loadRecordings();
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function summarize(): Promise<void> {
    const group = activeGroup.value;
    if (!group) return;
    const settings = useSettingsStore();

    processing.value = true;
    error.value = null;
    try {
      await groupApi.summarize(group.id, {
        provider: settings.summaryProvider,
        endpoint: settings.activeEndpoint,
        model: settings.activeModel,
        targetLanguage: settings.outputLanguage,
      });
      await loadGroups();
    } catch (cause) {
      error.value = String(cause);
    } finally {
      processing.value = false;
    }
  }

  async function processAll(): Promise<void> {
    const group = activeGroup.value;
    if (!group) return;

    const models = useModelsStore();
    const settings = useSettingsStore();
    const pipeline = usePipelineStore();
    const sttModel = models.downloadedStt;

    if (!sttModel) {
      error.value = "Download a speech-to-text model before processing.";
      return;
    }

    processing.value = true;
    error.value = null;
    try {
      for (const recording of recordings.value) {
        if (recording.transcript_raw) continue;
        processLabel.value = recording.file_name;
        pipeline.begin(recording.id, recording.file_name);
        try {
          await sttApi.transcribe(
            recording.id,
            sttModel.path,
            settings.audioLanguage,
            false,
            settings.useGpu,
          );
          pipeline.succeed();
        } catch (cause) {
          pipeline.fail(String(cause));
          throw cause;
        }
        await loadRecordings();
      }

      for (const recording of recordings.value) {
        if (recording.summary_markdown) continue;
        processLabel.value = recording.file_name;
        pipeline.begin(recording.id, recording.file_name);
        try {
          await llmApi.summarizeRecording(recording.id, {
            provider: settings.summaryProvider,
            endpoint: settings.activeEndpoint,
            model: settings.activeModel,
            targetLanguage: settings.outputLanguage,
          });
          pipeline.succeed();
        } catch (cause) {
          pipeline.fail(String(cause));
          throw cause;
        }
        await loadRecordings();
      }

      await summarize();
    } catch (cause) {
      error.value = String(cause);
    } finally {
      processing.value = false;
      processLabel.value = null;
    }
  }

  function dayLabel(recording: Recording): string {
    const date = recording.recorded_at
      ? new Date(recording.recorded_at * 1000)
      : new Date(recording.created_at.replace(" ", "T") + "Z");
    if (Number.isNaN(date.getTime())) return "Undated";
    const pad = (value: number) => String(value).padStart(2, "0");
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  }

  async function autoGroupByDay(candidates: Recording[]): Promise<number> {
    error.value = null;
    let assigned = 0;
    try {
      for (const recording of candidates) {
        if (recording.group_id) continue;
        const name = dayLabel(recording);
        let group = groups.value.find((candidate) => candidate.name === name);
        if (!group) {
          group = await groupApi.create(name);
          groups.value.push(group);
        }
        await groupApi.assign(recording.id, group.id);
        assigned += 1;
      }
      await loadGroups();
      if (activeGroupId.value) {
        await loadRecordings();
      }
    } catch (cause) {
      error.value = String(cause);
    }
    return assigned;
  }

  return {
    groups,
    recordings,
    activeGroupId,
    activeGroup,
    summary,
    loading,
    processing,
    processLabel,
    error,
    loadGroups,
    loadRecordings,
    selectGroup,
    createGroup,
    renameGroup,
    removeGroup,
    assign,
    summarize,
    processAll,
    dayLabel,
    autoGroupByDay,
  };
});
