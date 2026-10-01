import { invoke } from "@tauri-apps/api/core";
import type { ModelStatus } from "@/types";

export const modelApi = {
  list: () => invoke<ModelStatus[]>("list_models"),

  download: (url: string, filename: string) =>
    invoke<string>("download_model", { url, filename }),

  remove: (filename: string) => invoke<void>("delete_model", { filename }),
};
