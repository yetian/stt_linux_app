import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { projectApi } from "@/api/projects";
import type { NewRecording, Project, Recording, Tag } from "@/types";

export const useProjectsStore = defineStore("projects", () => {
  const projects = ref<Project[]>([]);
  const recordings = ref<Recording[]>([]);
  const activeProjectId = ref<string | null>(null);
  const activeRecordingId = ref<string | null>(null);
  const tagsByRecording = ref<Record<string, Tag[]>>({});
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

  async function renameProject(projectId: string, name: string): Promise<void> {
    error.value = null;
    try {
      const updated = await projectApi.rename(projectId, name);
      const index = projects.value.findIndex((p) => p.id === projectId);
      if (index !== -1) {
        projects.value[index] = updated;
      }
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function renameRecording(
    recordingId: string,
    fileName: string,
  ): Promise<void> {
    error.value = null;
    try {
      const updated = await projectApi.renameRecording(recordingId, fileName);
      const index = recordings.value.findIndex((r) => r.id === recordingId);
      if (index !== -1) {
        recordings.value[index] = updated;
      }
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function loadTags(recordingId: string): Promise<void> {
    try {
      const tags = await projectApi.listTags(recordingId);
      tagsByRecording.value = { ...tagsByRecording.value, [recordingId]: tags };
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function addTag(recordingId: string, tagName: string): Promise<void> {
    const name = tagName.trim();
    if (name === "") return;
    error.value = null;
    try {
      await projectApi.addTag(recordingId, name);
      await loadTags(recordingId);
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function removeTag(recordingId: string, tagId: number): Promise<void> {
    error.value = null;
    try {
      await projectApi.removeTag(tagId);
      await loadTags(recordingId);
    } catch (cause) {
      error.value = String(cause);
    }
  }

  function recordingTags(recordingId: string): Tag[] {
    return tagsByRecording.value[recordingId] ?? [];
  }

  const AUDIO_EXTENSIONS = [".mp3", ".wav", ".m4a", ".aac", ".flac"];

  function isAudioPath(path: string): boolean {
    const lower = path.toLowerCase();
    return AUDIO_EXTENSIONS.some((extension) => lower.endsWith(extension));
  }

  function baseName(path: string): string {
    const parts = path.split("/");
    return parts[parts.length - 1] ?? path;
  }

  async function addRecordingsFromPaths(paths: string[]): Promise<number> {
    const audioPaths = paths.filter(isAudioPath);

    for (const path of audioPaths) {
      await addRecording({
        file_name: baseName(path),
        source_path: path,
        project_id: activeProjectId.value,
        audio_duration_secs: null,
      });
    }

    if (audioPaths.length > 0) {
      await loadProjects();
    }

    return audioPaths.length;
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
    tagsByRecording,
    loading,
    error,
    loadProjects,
    loadRecordings,
    selectProject,
    createProject,
    removeProject,
    renameProject,
    addRecording,
    addRecordingsFromPaths,
    removeRecording,
    renameRecording,
    assignRecording,
    loadTags,
    addTag,
    removeTag,
    recordingTags,
    search,
    setActiveRecording,
  };
});
