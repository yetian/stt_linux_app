import { invoke } from "@tauri-apps/api/core";
import type { AppPaths } from "@/types";

export const systemApi = {
  getPaths: () => invoke<AppPaths>("get_app_paths"),
};
