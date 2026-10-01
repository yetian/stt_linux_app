import { invoke } from "@tauri-apps/api/core";
import type { ConnectedDevice } from "@/types";

export const deviceApi = {
  getConnectedDevice: () =>
    invoke<ConnectedDevice | null>("get_connected_device"),
};
