<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import DeviceFiles from "@/components/DeviceFiles.vue";
import ProjectTree from "@/components/ProjectTree.vue";
import RecordingList from "@/components/RecordingList.vue";
import SearchBar from "@/components/SearchBar.vue";
import { useProjectsStore } from "@/stores/projects";
import { useUiStore, type AppView } from "@/stores/ui";

const { t } = useI18n();
const projects = useProjectsStore();
const ui = useUiStore();

const query = ref("");
let timer: ReturnType<typeof setTimeout> | undefined;

watch(query, (value) => {
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    void projects.search(value);
  }, 300);
});

onBeforeUnmount(() => {
  if (timer) clearTimeout(timer);
});

const navItems: { view: AppView; labelKey: string; icon: string }[] = [
  { view: "workspace", labelKey: "nav.workspace", icon: "🖥️" },
  { view: "models", labelKey: "nav.models", icon: "🧠" },
  { view: "settings", labelKey: "nav.settings", icon: "⚙️" },
];
</script>

<template>
  <aside class="flex flex-col gap-4 border-r border-base-800 bg-base-900 px-4 py-5">
    <div class="flex items-center gap-3">
      <div class="grid size-9 place-items-center rounded-xl bg-accent-500/15 text-lg leading-none text-accent-400">
        🎙️
      </div>
      <div class="min-w-0">
        <p class="truncate text-sm font-semibold text-slate-100">{{ t("app.name") }}</p>
        <p class="truncate text-[11px] text-base-500">{{ t("app.tagline") }}</p>
      </div>
    </div>

    <SearchBar v-model="query" />

    <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto pr-1">
      <ProjectTree />
      <DeviceFiles />
      <RecordingList />
    </div>

    <div class="flex flex-col gap-1 border-t border-base-800 pt-4">
      <button
        v-for="item in navItems"
        :key="item.view"
        type="button"
        class="rounded-lg px-3 py-2 text-left text-sm transition-colors"
        :class="
          ui.activeView === item.view
            ? 'bg-base-800 text-slate-100'
            : 'text-slate-400 hover:bg-base-850 hover:text-slate-200'
        "
        @click="ui.setView(item.view)"
      >
        <span class="mr-2">{{ item.icon }}</span>{{ t(item.labelKey) }}
      </button>
    </div>
  </aside>
</template>
