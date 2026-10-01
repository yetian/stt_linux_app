import { invoke } from "@tauri-apps/api/core";

export type SettingsMap = Record<string, string>;

export const settingsApi = {
  load: () => invoke<SettingsMap>("load_settings"),
  save: (settings: SettingsMap) => invoke<void>("save_settings", { settings }),
};
