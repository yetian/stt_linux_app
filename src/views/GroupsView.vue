<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import SummaryPanel from "@/components/SummaryPanel.vue";
import TextInputDialog from "@/components/TextInputDialog.vue";
import { projectApi } from "@/api/projects";
import { useGroupsStore } from "@/stores/groups";
import { useModelsStore } from "@/stores/models";
import { useProjectsStore } from "@/stores/projects";
import { useUiStore } from "@/stores/ui";
import type { Recording } from "@/types";

const { t } = useI18n();
const groups = useGroupsStore();
const models = useModelsStore();
const projects = useProjectsStore();
const ui = useUiStore();

const STATUS_COLOR: Record<string, string> = {
  pending: "bg-base-500",
  transcribing: "bg-amber-400",
  summarizing: "bg-sky-400",
  completed: "bg-emerald-400",
  failed: "bg-rose-500",
};

const allRecordings = ref<Recording[]>([]);
const createOpen = ref(false);
const renameOpen = ref(false);
const deleteOpen = ref(false);
const addSelection = ref("");

const canProcess = computed(() => models.downloadedStt !== null);

const addable = computed(() =>
  allRecordings.value.filter(
    (recording) => recording.group_id !== groups.activeGroupId,
  ),
);

function pad(value: number): string {
  return String(value).padStart(2, "0");
}

function todayLabel(): string {
  const now = new Date();
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

async function reloadAll(): Promise<void> {
  allRecordings.value = await projectApi.listRecordings(null);
}

onMounted(async () => {
  await groups.loadGroups();
  await reloadAll();
  if (!groups.activeGroupId && groups.groups.length > 0) {
    await groups.selectGroup(groups.groups[0].id);
  }
});

function select(groupId: string): void {
  void groups.selectGroup(groupId);
}

async function submitCreate(name: string): Promise<void> {
  createOpen.value = false;
  await groups.createGroup(name);
  await reloadAll();
}

async function submitRename(name: string): Promise<void> {
  const group = groups.activeGroup;
  renameOpen.value = false;
  if (group) await groups.renameGroup(group.id, name);
}

async function confirmDelete(): Promise<void> {
  const group = groups.activeGroup;
  deleteOpen.value = false;
  if (group) await groups.removeGroup(group.id);
  await reloadAll();
}

async function addSelected(): Promise<void> {
  if (!groups.activeGroupId || addSelection.value === "") return;
  await groups.assign(addSelection.value, groups.activeGroupId);
  addSelection.value = "";
  await reloadAll();
}

async function removeMember(recordingId: string): Promise<void> {
  await groups.assign(recordingId, null);
  await reloadAll();
}

async function openRecording(recordingId: string): Promise<void> {
  await projects.selectProject(null);
  projects.setActiveRecording(recordingId);
  ui.setView("workspace");
}

async function autoGroup(): Promise<void> {
  await groups.autoGroupByDay(allRecordings.value);
  await reloadAll();
}
</script>

<template>
  <div class="mx-auto flex h-full max-w-6xl flex-col gap-5 px-6 py-6">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 class="text-lg font-semibold text-slate-100">
          <i class="fa-solid fa-layer-group mr-2 text-accent-400"></i>{{ t("groups.title") }}
        </h1>
        <p class="mt-0.5 text-xs text-base-500">{{ t("groups.subtitle") }}</p>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <button
          type="button"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:opacity-40"
          :disabled="groups.processing"
          @click="autoGroup"
        >
          <i class="fa-solid fa-wand-magic-sparkles mr-1"></i>{{ t("groups.autoGroup") }}
        </button>
        <button
          type="button"
          class="rounded-lg bg-accent-500 px-3 py-1.5 text-xs font-medium text-base-950 transition-colors hover:bg-accent-400"
          @click="createOpen = true"
        >
          <i class="fa-solid fa-plus mr-1"></i>{{ t("groups.new") }}
        </button>
      </div>
    </div>

    <p
      v-if="groups.error"
      class="rounded-lg border border-rose-500/40 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
    >
      <i class="fa-solid fa-triangle-exclamation mr-1"></i>{{ groups.error }}
    </p>

    <div class="grid min-h-0 flex-1 gap-5 lg:grid-cols-[16rem_1fr]">
      <aside
        class="flex max-h-full min-h-0 flex-col gap-1 overflow-y-auto rounded-2xl border border-base-800 bg-base-900 p-3"
      >
        <p v-if="groups.groups.length === 0" class="px-2 py-6 text-center text-xs text-base-500">
          {{ t("groups.empty") }}
        </p>
        <button
          v-for="group in groups.groups"
          :key="group.id"
          type="button"
          class="flex flex-col rounded-lg px-3 py-2 text-left transition-colors"
          :class="groups.activeGroupId === group.id ? 'bg-base-800' : 'hover:bg-base-850'"
          @click="select(group.id)"
        >
          <span class="truncate text-sm text-slate-200">{{ group.name }}</span>
          <span class="mt-0.5 text-[10px] text-base-500">
            {{ t("groups.count", { count: group.recording_count }) }}
          </span>
        </button>
      </aside>

      <section
        v-if="groups.activeGroup"
        class="flex min-h-0 flex-col gap-4 overflow-y-auto rounded-2xl border border-base-800 bg-base-900 p-5"
      >
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex min-w-0 items-center gap-2">
            <p class="truncate text-base font-semibold text-slate-100">
              {{ groups.activeGroup.name }}
            </p>
            <button
              type="button"
              class="shrink-0 text-base-500 transition-colors hover:text-accent-400"
              :title="t('groups.renameTitle')"
              @click="renameOpen = true"
            >
              <i class="fa-solid fa-pen"></i>
            </button>
            <button
              type="button"
              class="shrink-0 text-base-500 transition-colors hover:text-rose-400"
              :title="t('groups.deleteTitle')"
              @click="deleteOpen = true"
            >
              <i class="fa-solid fa-trash"></i>
            </button>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <button
              type="button"
              class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:cursor-not-allowed disabled:opacity-40"
              :disabled="groups.processing || !canProcess || groups.recordings.length === 0"
              @click="groups.processAll()"
            >
              <i class="fa-solid fa-play mr-1"></i>{{ t("groups.processAll") }}
            </button>
            <button
              type="button"
              class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:cursor-not-allowed disabled:opacity-40"
              :disabled="groups.processing || groups.recordings.length === 0"
              @click="groups.summarize()"
            >
              <i class="fa-solid fa-file-lines mr-1"></i>{{ t("groups.summarize") }}
            </button>
          </div>
        </div>

        <p v-if="groups.processing" class="text-xs text-accent-400">
          <i class="fa-solid fa-spinner fa-spin mr-1"></i>
          {{ groups.processLabel ? t("groups.processing", { name: groups.processLabel }) : t("groups.summarizing") }}
        </p>

        <div class="flex flex-col gap-2">
          <div class="flex flex-wrap items-center gap-2">
            <select v-model="addSelection" class="select-field max-w-72">
              <option value="">{{ t("groups.addPlaceholder") }}</option>
              <option v-for="recording in addable" :key="recording.id" :value="recording.id">
                {{ recording.file_name }}
              </option>
            </select>
            <button
              type="button"
              class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 disabled:opacity-40"
              :disabled="addSelection === '' || groups.processing"
              @click="addSelected"
            >
              <i class="fa-solid fa-plus mr-1"></i>{{ t("groups.add") }}
            </button>
          </div>

          <p v-if="groups.recordings.length === 0" class="text-xs text-base-500">
            {{ t("groups.noMembers") }}
          </p>
          <ul v-else class="flex flex-col gap-1">
            <li
              v-for="recording in groups.recordings"
              :key="recording.id"
              class="flex items-center justify-between gap-3 rounded-lg bg-base-850 px-3 py-1.5"
            >
              <button
                type="button"
                class="flex min-w-0 items-center gap-2 text-left"
                :title="t('groups.open')"
                @click="openRecording(recording.id)"
              >
                <span
                  class="size-1.5 shrink-0 rounded-full"
                  :class="STATUS_COLOR[recording.status] ?? 'bg-base-500'"
                />
                <span class="truncate text-xs text-slate-300">{{ recording.file_name }}</span>
                <span class="shrink-0 text-[10px] text-base-500">
                  {{ t(`status.${recording.status}`) }}
                </span>
              </button>
              <button
                type="button"
                class="shrink-0 text-base-500 transition-colors hover:text-rose-400"
                :title="t('groups.removeMember')"
                @click="removeMember(recording.id)"
              >
                <i class="fa-solid fa-xmark"></i>
              </button>
            </li>
          </ul>
        </div>

        <div class="border-t border-base-800 pt-4">
          <p class="mb-3 text-xs font-medium tracking-wide text-base-500 uppercase">
            {{ t("groups.summaryTitle") }}
          </p>
          <SummaryPanel :markdown="groups.summary" :file-name="groups.activeGroup.name" />
        </div>
      </section>

      <section
        v-else
        class="grid place-items-center rounded-2xl border border-base-800 bg-base-900 px-6 py-16 text-center"
      >
        <div class="max-w-sm">
          <div class="mx-auto grid size-12 place-items-center rounded-full bg-base-800 text-base-500">
            <i class="fa-solid fa-layer-group"></i>
          </div>
          <p class="mt-4 text-sm font-medium text-slate-300">{{ t("groups.noSelection") }}</p>
          <p class="mt-1 text-xs text-base-500">{{ t("groups.noSelectionHint") }}</p>
        </div>
      </section>
    </div>

    <TextInputDialog
      :open="createOpen"
      :title="t('groups.createTitle')"
      :label="t('groups.nameLabel')"
      :initial-value="todayLabel()"
      @confirm="submitCreate"
      @cancel="createOpen = false"
    />

    <TextInputDialog
      :open="renameOpen"
      :title="t('groups.renameTitle')"
      :label="t('groups.nameLabel')"
      :initial-value="groups.activeGroup?.name"
      @confirm="submitRename"
      @cancel="renameOpen = false"
    />

    <ConfirmDialog
      :open="deleteOpen"
      :title="t('groups.deleteTitle')"
      :message="t('groups.deleteMessage', { name: groups.activeGroup?.name })"
      :confirm-label="t('common.delete')"
      danger
      @confirm="confirmDelete"
      @cancel="deleteOpen = false"
    />
  </div>
</template>
