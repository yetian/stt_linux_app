<script setup lang="ts">
import { onMounted } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import AppSidebar from "@/components/layout/AppSidebar.vue";
import AppHeader from "@/components/layout/AppHeader.vue";
import ModelsView from "@/views/ModelsView.vue";
import SettingsView from "@/views/SettingsView.vue";
import WorkspaceView from "@/views/WorkspaceView.vue";
import { useDeviceStore } from "@/stores/device";
import { useModelsStore } from "@/stores/models";
import { useProjectsStore } from "@/stores/projects";
import { useUiStore } from "@/stores/ui";

const projects = useProjectsStore();
const device = useDeviceStore();
const models = useModelsStore();
const ui = useUiStore();

onMounted(async () => {
  await device.initialize();
  await models.initialize();
  await projects.loadProjects();
  await projects.loadRecordings();

  await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type !== "drop") return;

    const added = await projects.addRecordingsFromPaths(event.payload.paths);
    if (added > 0) {
      ui.setView("workspace");
    }
  });
});
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-base-950 text-slate-200">
    <AppSidebar class="hidden w-72 shrink-0 md:flex" />
    <div class="flex min-w-0 flex-1 flex-col">
      <AppHeader />
      <main class="min-h-0 flex-1 overflow-hidden">
        <WorkspaceView v-if="ui.activeView === 'workspace'" />
        <ModelsView v-else-if="ui.activeView === 'models'" />
        <SettingsView v-else />
      </main>
    </div>
  </div>
</template>
