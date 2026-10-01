import { defineStore } from "pinia";
import { ref } from "vue";

export type AppView = "workspace" | "models" | "settings";

export const useUiStore = defineStore("ui", () => {
  const activeView = ref<AppView>("workspace");

  function setView(view: AppView): void {
    activeView.value = view;
  }

  return {
    activeView,
    setView,
  };
});
