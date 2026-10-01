import { invoke } from "@tauri-apps/api/core";
import type { TranscriptSegment } from "@/types";

export const sttApi = {
  transcribe: (
    recordingId: string,
    modelPath: string,
    language?: string | null,
    translate = false,
    useGpu = true,
  ) =>
    invoke<TranscriptSegment[]>("transcribe_recording", {
      recordingId,
      modelPath,
      language: language ?? null,
      translate,
      useGpu,
    }),
};
