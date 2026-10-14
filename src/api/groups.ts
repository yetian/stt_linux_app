import { invoke } from "@tauri-apps/api/core";
import type { Group, Recording } from "@/types";
import type { SummarizeParams } from "@/api/llm";

export const groupApi = {
  list: () => invoke<Group[]>("list_groups"),

  create: (name: string) => invoke<Group>("create_group", { name }),

  rename: (groupId: string, name: string) =>
    invoke<Group>("rename_group", { groupId, name }),

  remove: (groupId: string) => invoke<void>("delete_group", { groupId }),

  listRecordings: (groupId: string) =>
    invoke<Recording[]>("list_group_recordings", { groupId }),

  assign: (recordingId: string, groupId: string | null) =>
    invoke<void>("assign_recording_to_group", { recordingId, groupId }),

  summarize: (groupId: string, params: SummarizeParams) =>
    invoke<string>("summarize_group", {
      groupId,
      provider: params.provider,
      endpoint: params.endpoint,
      model: params.model,
      targetLanguage: params.targetLanguage,
    }),
};
