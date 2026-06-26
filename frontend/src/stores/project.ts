import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  Task,
  TaskType,
  TaskStats,
  ClassificationMode,
} from "@/utils/taskTypes";
import type { ClassDefinition, ImageInfo } from "@/utils/types";

export interface ProjectData {
  tasks: Task[];
}

const TASKS_FILE = "projects.json";

function genId(): string {
  return crypto.randomUUID();
}

function now(): string {
  return new Date().toISOString();
}

export const useProjectStore = defineStore("project", () => {
  // ==================== 状态 ====================
  const tasks = ref<Task[]>([]);
  const currentTaskId = ref<string | null>(null);
  const currentTask = computed(() =>
    tasks.value.find((t) => t.id === currentTaskId.value) ?? null
  );

  // 首页筛选状态（存store避免返回时丢失）
  const filterText = ref("");
  const filterType = ref("");
  const filterSort = ref("newest");

  // ==================== 持久化 ====================

  async function loadProject() {
    try {
      const data: ProjectData = await invoke("load_project_list", {
        fileName: TASKS_FILE,
      });
      tasks.value = data.tasks ?? [];
    } catch {
      tasks.value = [];
    }
  }

  async function saveProject() {
    try {
      await invoke("save_project_list", {
        fileName: TASKS_FILE,
        data: { tasks: tasks.value },
      });
    } catch (e) {
      console.error("保存任务列表失败:", e);
    }
  }

  // ==================== 任务 CRUD ====================

  async function createTask(name: string, taskType: TaskType, imageFolder: string, classificationMode?: ClassificationMode) {
    const id = genId();
    const cls: ClassDefinition[] = getDefaultClasses(taskType);

    // 加载图片列表
    let images: ImageInfo[] = [];
    try {
      const folderInfo = await invoke<{ folder_path: string; images: ImageInfo[] }>(
        "load_folder",
        { folderPath: imageFolder }
      );
      images = folderInfo.images;
    } catch (e) {
      console.error("加载图片目录失败:", e);
    }

    const config = taskType === "classification" ? { classification_mode: classificationMode ?? "single" } : undefined;

    const task: Task = {
      id,
      name: name.trim(),
      task_type: taskType,
      config,
      created_at: now(),
      updated_at: now(),
      image_folder: imageFolder,
      classes: cls,
      stats: {
        total_images: images.length,
        annotated_images: 0,
        total_annotations: 0,
      },
    };

    tasks.value.push(task);
    await saveProject();
    return task;
  }

  async function deleteTask(id: string) {
    const idx = tasks.value.findIndex((t) => t.id === id);
    if (idx !== -1) {
      tasks.value.splice(idx, 1);
      if (currentTaskId.value === id) {
        currentTaskId.value = null;
      }
      await saveProject();
    }
  }

  async function updateTaskStats(id: string, stats: Partial<TaskStats>) {
    const task = tasks.value.find((t) => t.id === id);
    if (task) {
      Object.assign(task.stats, stats);
      task.updated_at = now();
      await saveProject();
    }
  }

  async function updateTaskClasses(id: string, classes: ClassDefinition[]) {
    const task = tasks.value.find((t) => t.id === id);
    if (task) {
      task.classes = classes;
      task.updated_at = now();
      await saveProject();
    }
  }

  // ==================== 导航 ====================

  function openTask(id: string) {
    currentTaskId.value = id;
  }

  function closeTask() {
    currentTaskId.value = null;
  }

  // ==================== 内部 ====================

  function getDefaultClasses(taskType: TaskType): ClassDefinition[] {
    const configs: Record<TaskType, ClassDefinition[]> = {
      classification: [
        mkCls(0, "正类", "#f97316"),
        mkCls(1, "负类", "#6366f1"),
      ],
      detection: [
        mkCls(0, "物体", "#38bdf8"),
        mkCls(1, "车辆", "#3b82f6"),
        mkCls(2, "人", "#4ade80"),
      ],
      rotated_detection: [
        mkCls(0, "物体", "#f97316"),
        mkCls(1, "车辆", "#3b82f6"),
      ],
      keypoint: [
        mkCls(0, "车牌", "#fbbf24", ["top_left", "top_right", "bottom_right", "bottom_left"]),
        mkCls(1, "人脸", "#f472b6", ["left_eye", "right_eye", "nose", "left_mouth", "right_mouth"]),
      ],
      segmentation: [
        mkCls(0, "物体", "#38bdf8"),
        mkCls(1, "区域", "#4ade80"),
      ],
      ocr: [
        mkCls(0, "文字", "#eab308"),
      ],
    };
    return configs[taskType] ?? [];
  }

  return {
    tasks,
    currentTaskId,
    currentTask,
    filterText,
    filterType,
    filterSort,
    loadProject,
    saveProject,
    createTask,
    deleteTask,
    updateTaskStats,
    updateTaskClasses,
    openTask,
    closeTask,
  };
});

function mkCls(id: number, name: string, color: string, kpnames?: string[]): ClassDefinition {
  const cls: ClassDefinition = { id, name, color };
  if (kpnames) {
    cls.keypoint_names = kpnames;
    cls.keypoint_count = kpnames.length;
  }
  return cls;
}
