import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface ModelConfig {
  id: string;
  name: string;
  task_type: string;
  model_path: string;
  ocr_det?: string;
  ocr_cls?: string;
  ocr_rec?: string;
  ocr_dict?: string;
  runtime: "gpu" | "cpu";
  fp16: boolean;
  created_at: string;
}

const MODELS_FILE = "models.json";

export const useModelStore = defineStore("models", () => {
  const models = ref<ModelConfig[]>([]);
  const showModal = ref(false);

  async function load() {
    try {
      const data = await invoke<{ models: ModelConfig[] }>("load_project_list", {
        fileName: MODELS_FILE,
      });
      models.value = data.models ?? [];
    } catch {
      models.value = [];
    }
  }

  async function save() {
    try {
      await invoke("save_project_list", {
        fileName: MODELS_FILE,
        data: { models: models.value },
      });
    } catch (e) {
      console.error("保存模型配置失败:", e);
    }
  }

  function addModel(config: ModelConfig) {
    models.value.push(config);
    save();
  }

  function removeModel(id: string) {
    models.value = models.value.filter((m) => m.id !== id);
    save();
  }

  function openModal() { showModal.value = true; }
  function closeModal() { showModal.value = false; }

  return {
    models,
    showModal,
    load,
    save,
    addModel,
    removeModel,
    openModal,
    closeModal,
  };
});