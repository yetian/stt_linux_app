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

const AUDIO_EXTENSIONS = [".mp3", ".wav", ".m4a", ".aac", ".flac"];

function isAudio(path: string): boolean {
  const lower = path.toLowerCase();
  return AUDIO_EXTENSIONS.some((extension) => lower.endsWith(extension));
}

function baseName(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1] ?? path;
}

onMounted(async () => {
  await device.initialize();
  await models.initialize();
  await projects.loadProjects();
  await projects.loadRecordings();

  await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type !== "drop") return;

    const paths = event.payload.paths.filter(isAudio);
    if (paths.length === 0) return;

    for (const path of paths) {
      await projects.addRecording({
        file_name: baseName(path),
        source_path: path,
        project_id: projects.activeProjectId,
        audio_duration_secs: null,
      });
    }

    await projects.loadProjects();
    ui.setView("workspace");
  });
});
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-base-950 text-slate-200">
    <AppSidebar class="hidden w-72 shrink-0 md:flex" />
    <div class="flex min-w-0 flex-1 flex-col">
      <AppHeader />
      <main class="min-h-0 flex-1 overflow-y-auto">
        <WorkspaceView v-if="ui.activeView === 'workspace'" />
        <ModelsView v-else-if="ui.activeView === 'models'" />
        <SettingsView v-else />
      </main>
    </div>
  </div>
</template>
