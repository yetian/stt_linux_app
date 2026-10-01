import { invoke } from "@tauri-apps/api/core";
import type { DiarizedSegment, TranscriptSegment } from "@/types";

export const diarizationApi = {
  diarize: (
    recordingId: string,
    embeddingModelPath: string,
    segments: TranscriptSegment[],
    threshold = 0.35,
  ) =>
    invoke<DiarizedSegment[]>("diarize_recording", {
      recordingId,
      embeddingModelPath,
      segments,
      threshold,
    }),
};
