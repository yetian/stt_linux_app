import { invoke } from "@tauri-apps/api/core";
import type { AppPaths, GpuInfo } from "@/types";

export const systemApi = {
  getPaths: () => invoke<AppPaths>("get_app_paths"),
  getGpuInfo: () => invoke<GpuInfo | null>("get_gpu_info"),
};
