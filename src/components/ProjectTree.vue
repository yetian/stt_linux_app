<script setup lang="ts">
import { nextTick, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useProjectsStore } from "@/stores/projects";

const { t } = useI18n();
const projects = useProjectsStore();

const creating = ref(false);
const draftName = ref("");
const nameInput = ref<HTMLInputElement | null>(null);

async function startCreate(): Promise<void> {
  creating.value = true;
  draftName.value = "";
  await nextTick();
  nameInput.value?.focus();
}

async function confirmCreate(): Promise<void> {
  const name = draftName.value.trim();
  creating.value = false;
  if (name === "") return;
  await projects.createProject(name);
}

function cancelCreate(): void {
  creating.value = false;
  draftName.value = "";
}
</script>

<template>
  <div class="flex flex-col gap-1">
    <div class="flex items-center justify-between px-1 pb-1">
      <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
        📁 {{ t("nav.projects") }}
      </span>
      <button
        type="button"
        class="rounded-md px-1.5 text-base-500 transition-colors hover:bg-base-800 hover:text-slate-200"
        :title="t('projects.new')"
        @click="startCreate"
      >
        ➕
      </button>
    </div>

    <div v-if="creating" class="px-1 pb-1">
      <input
        ref="nameInput"
        v-model="draftName"
        type="text"
        :placeholder="t('projects.new')"
        class="w-full rounded-lg border border-accent-500/50 bg-base-850 px-2.5 py-1.5 text-sm text-slate-100 outline-none placeholder:text-base-500"
        @keydown.enter.prevent="confirmCreate"
        @keydown.esc.prevent="cancelCreate"
        @blur="confirmCreate"
      />
    </div>

    <button
      type="button"
      class="flex items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors"
      :class="
        projects.activeProjectId === null
          ? 'bg-base-800 text-slate-100'
          : 'text-slate-400 hover:bg-base-850 hover:text-slate-200'
      "
      @click="projects.selectProject(null)"
    >
      <span class="truncate">🗂️ {{ t("nav.allRecordings") }}</span>
      <span class="ml-2 font-mono text-[11px] text-base-500">
        {{ projects.totalRecordings }}
      </span>
    </button>

    <p v-if="projects.projects.length === 0" class="px-3 py-2 text-xs text-base-500">
      {{ t("projects.empty") }}
    </p>

    <button
      v-for="project in projects.projects"
      :key="project.id"
      type="button"
      class="flex items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors"
      :class="
        projects.activeProjectId === project.id
          ? 'bg-base-800 text-slate-100'
          : 'text-slate-400 hover:bg-base-850 hover:text-slate-200'
      "
      @click="projects.selectProject(project.id)"
    >
      <span class="truncate">📁 {{ project.name }}</span>
      <span class="ml-2 font-mono text-[11px] text-base-500">
        {{ project.recording_count }}
      </span>
    </button>
  </div>
</template>
