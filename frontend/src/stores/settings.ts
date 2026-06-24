import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface AppSettings {
  accent_color: string;
  annotation_line_width: string;
  default_annotation_color: string;
  show_labels: boolean;
  status_bar_visible: boolean;
  dense_mode: boolean;
}

const ACCENT_PRESETS = [
  "#f97316", "#ef4444", "#ec4899", "#a855f7",
  "#6366f1", "#3b82f6", "#06b6d4", "#14b8a6",
  "#22c55e", "#84cc16", "#eab308", "#78716c",
];

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>({
    accent_color: "#f97316",
    annotation_line_width: "medium",
    default_annotation_color: "#38bdf8",
    show_labels: true,
    status_bar_visible: true,
    dense_mode: false,
  });

  const showModal = ref(false);

  function openModal() { showModal.value = true; }
  function closeModal() { showModal.value = false; }

  async function load() {
    try {
      const s = await invoke<AppSettings>("load_settings");
      settings.value = { ...settings.value, ...s };
    } catch {
      // use defaults
    }
  }

  async function save() {
    try {
      await invoke("save_settings", { settings: settings.value });
    } catch (e) {
      console.error("Save settings failed", e);
    }
  }

  watch(settings, () => { save(); }, { deep: true });

  function setAccent(color: string) {
    settings.value.accent_color = color;
  }

  function reset() {
    settings.value = {
      accent_color: "#f97316",
      annotation_line_width: "medium",
      default_annotation_color: "#38bdf8",
      show_labels: true,
      status_bar_visible: true,
      dense_mode: false,
    };
  }

  return {
    settings,
    showModal,
    ACCENT_PRESETS,
    openModal,
    closeModal,
    load,
    save,
    setAccent,
    reset,
  };
});
