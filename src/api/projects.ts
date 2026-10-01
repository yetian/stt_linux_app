import { invoke } from "@tauri-apps/api/core";
import type { NewRecording, Project, Recording, Tag } from "@/types";

export const projectApi = {
  list: () => invoke<Project[]>("list_projects"),

  create: (name: string, description?: string | null) =>
    invoke<Project>("create_project", {
      name,
      description: description ?? null,
    }),

  remove: (projectId: string) =>
    invoke<void>("delete_project", { projectId }),

  rename: (projectId: string, name: string) =>
    invoke<Project>("rename_project", { projectId, name }),

  listRecordings: (projectId?: string | null) =>
    invoke<Recording[]>("list_recordings", { projectId: projectId ?? null }),

  search: (
    queryKeyword?: string | null,
    projectId?: string | null,
    tagFilter?: string | null,
  ) =>
    invoke<Recording[]>("search_recordings", {
      queryKeyword: queryKeyword ?? null,
      projectId: projectId ?? null,
      tagFilter: tagFilter ?? null,
    }),

  addRecording: (input: NewRecording) =>
    invoke<Recording>("add_recording", { input }),

  assignToProject: (recordingId: string, projectId: string | null) =>
    invoke<void>("assign_recording_to_project", { recordingId, projectId }),

  removeRecording: (recordingId: string) =>
    invoke<void>("delete_recording", { recordingId }),

  renameRecording: (recordingId: string, fileName: string) =>
    invoke<Recording>("rename_recording", { recordingId, fileName }),

  updateStatus: (recordingId: string, status: string) =>
    invoke<void>("update_recording_status", { recordingId, status }),

  saveTranscript: (recordingId: string, transcript: string) =>
    invoke<void>("save_transcript", { recordingId, transcript }),

  saveSummary: (recordingId: string, summary: string) =>
    invoke<void>("save_summary", { recordingId, summary }),

  listTags: (recordingId: string) =>
    invoke<Tag[]>("list_tags", { recordingId }),

  addTag: (recordingId: string, tagName: string) =>
    invoke<void>("add_tag", { recordingId, tagName }),

  removeTag: (tagId: number) => invoke<void>("delete_tag", { tagId }),

  exportRecording: (recordingId: string) =>
    invoke<string>("export_recording", { recordingId }),
};
