import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { deviceApi } from "@/api/device";
import type { ConnectedDevice } from "@/types";

export const useDeviceStore = defineStore("device", () => {
  const device = ref<ConnectedDevice | null>(null);
  const initialized = ref(false);

  const isConnected = computed(() => device.value !== null);
  const mountPath = computed(() => device.value?.mount_path ?? null);
  const files = computed(() => device.value?.files ?? []);

  function setDevice(next: ConnectedDevice | null): void {
    device.value = next;
  }

  async function initialize(): Promise<void> {
    if (initialized.value) return;
    initialized.value = true;

    try {
      device.value = await deviceApi.getConnectedDevice();
    } catch {
      device.value = null;
    }

    const unlisteners: UnlistenFn[] = [
      await listen<ConnectedDevice>("usb-device-attached", (event) => {
        device.value = event.payload;
      }),
      await listen("usb-device-detached", () => {
        device.value = null;
      }),
    ];

    window.addEventListener("beforeunload", () => {
      for (const unlisten of unlisteners) unlisten();
    });
  }

  return {
    device,
    initialized,
    isConnected,
    mountPath,
    files,
    setDevice,
    initialize,
  };
});
