export type SummaryProvider = "ollama" | "openai";

export interface LlmModel {
  name: string;
  size_bytes: number;
}

export interface TranscriptSegment {
  start_ms: number;
  end_ms: number;
  text: string;
}

export interface DiarizedSegment {
  start_ms: number;
  end_ms: number;
  speaker: number;
  text: string;
}

export type ModelKind = "stt" | "diarization";

export interface ModelStatus {
  id: string;
  name: string;
  kind: ModelKind;
  size_mb: number;
  url: string;
  filename: string;
  downloaded: boolean;
  path: string;
}

export interface ModelDownloadProgress {
  filename: string;
  downloaded: number;
  total: number | null;
  done: boolean;
}

export type RecordingStatus =
  | "pending"
  | "transcribing"
  | "summarizing"
  | "completed"
  | "failed";

export interface Project {
  id: string;
  name: string;
  description: string | null;
  created_at: string;
  recording_count: number;
}

export interface Recording {
  id: string;
  project_id: string | null;
  file_name: string;
  source_path: string;
  audio_duration_secs: number | null;
  detected_language: string | null;
  status: RecordingStatus;
  transcript_raw: string | null;
  summary_markdown: string | null;
  transcript_segments: string | null;
  created_at: string;
}

export interface NewRecording {
  file_name: string;
  source_path: string;
  project_id: string | null;
  audio_duration_secs: number | null;
}

export interface Tag {
  id: number;
  recording_id: string;
  tag_name: string;
}

export interface GpuInfo {
  name: string;
  vram_mb: number;
}

export interface AppPaths {
  config_dir: string;
  data_dir: string;
  models_dir: string;
  outputs_dir: string;
}

export interface RecorderFile {
  path: string;
  name: string;
  size_bytes: number;
  modified_at: number | null;
  duration_secs: number | null;
}

export interface ConnectedDevice {
  mount_path: string;
  label: string;
  files: RecorderFile[];
}
