import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { projectApi } from "@/api/projects";
import type { NewRecording, Project, Recording } from "@/types";

export const useProjectsStore = defineStore("projects", () => {
  const projects = ref<Project[]>([]);
  const recordings = ref<Recording[]>([]);
  const activeProjectId = ref<string | null>(null);
  const activeRecordingId = ref<string | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const activeProject = computed(
    () => projects.value.find((p) => p.id === activeProjectId.value) ?? null,
  );

  const activeRecording = computed(
    () => recordings.value.find((r) => r.id === activeRecordingId.value) ?? null,
  );

  const totalRecordings = computed(() =>
    projects.value.reduce((sum, project) => sum + project.recording_count, 0),
  );

  async function loadProjects(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      projects.value = await projectApi.list();
    } catch (cause) {
      error.value = String(cause);
    } finally {
      loading.value = false;
    }
  }

  async function loadRecordings(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      recordings.value = await projectApi.listRecordings(activeProjectId.value);
    } catch (cause) {
      error.value = String(cause);
    } finally {
      loading.value = false;
    }
  }

  async function selectProject(projectId: string | null): Promise<void> {
    activeProjectId.value = projectId;
    activeRecordingId.value = null;
    await loadRecordings();
  }

  async function createProject(
    name: string,
    description?: string | null,
  ): Promise<Project | null> {
    error.value = null;
    try {
      const project = await projectApi.create(name, description ?? null);
      projects.value.push(project);
      return project;
    } catch (cause) {
      error.value = String(cause);
      return null;
    }
  }

  async function removeProject(projectId: string): Promise<void> {
    error.value = null;
    try {
      await projectApi.remove(projectId);
      projects.value = projects.value.filter((p) => p.id !== projectId);
      if (activeProjectId.value === projectId) {
        await selectProject(null);
      }
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function addRecording(input: NewRecording): Promise<Recording | null> {
    error.value = null;
    try {
      const recording = await projectApi.addRecording(input);
      recordings.value.unshift(recording);
      return recording;
    } catch (cause) {
      error.value = String(cause);
      return null;
    }
  }

  async function removeRecording(recordingId: string): Promise<void> {
    error.value = null;
    try {
      await projectApi.removeRecording(recordingId);
      recordings.value = recordings.value.filter((r) => r.id !== recordingId);
      if (activeRecordingId.value === recordingId) {
        activeRecordingId.value = null;
      }
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function assignRecording(
    recordingId: string,
    projectId: string | null,
  ): Promise<void> {
    error.value = null;
    try {
      await projectApi.assignToProject(recordingId, projectId);
      const recording = recordings.value.find((r) => r.id === recordingId);
      if (recording) {
        recording.project_id = projectId;
      }
      await loadProjects();
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function search(query: string, tag?: string | null): Promise<void> {
    error.value = null;
    if (query.trim() === "" && !tag) {
      await loadRecordings();
      return;
    }
    try {
      recordings.value = await projectApi.search(
        query,
        activeProjectId.value,
        tag ?? null,
      );
    } catch (cause) {
      error.value = String(cause);
    }
  }

  function setActiveRecording(recordingId: string | null): void {
    activeRecordingId.value = recordingId;
  }

  return {
    projects,
    recordings,
    activeProjectId,
    activeRecordingId,
    activeProject,
    activeRecording,
    totalRecordings,
    loading,
    error,
    loadProjects,
    loadRecordings,
    selectProject,
    createProject,
    removeProject,
    addRecording,
    removeRecording,
    assignRecording,
    search,
    setActiveRecording,
  };
});
