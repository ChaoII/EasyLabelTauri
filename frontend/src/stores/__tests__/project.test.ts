import { describe, it, expect, beforeEach, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useProjectStore } from "../project";
import type { Task } from "@/utils/taskTypes";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

describe("useProjectStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("初始状态", () => {
    const store = useProjectStore();
    expect(store.tasks).toEqual([]);
    expect(store.currentTaskId).toBeNull();
    expect(store.currentTask).toBeNull();
    expect(store.exportHistory).toEqual([]);
  });

  it("openTask / closeTask 导航", () => {
    const store = useProjectStore();
    const task: Task = {
      id: "task-1", name: "测试任务", task_type: "detection",
      image_folder: "/images", created_at: "2024-01-01", updated_at: "2024-01-01",
      classes: [{ id: 0, name: "物体", color: "#ff0000" }],
      stats: { total_images: 10, annotated_images: 3, total_annotations: 5 },
    };
    store.tasks = [task];

    store.openTask("task-1");
    expect(store.currentTaskId).toBe("task-1");
    expect(store.currentTask?.name).toBe("测试任务");

    store.closeTask();
    expect(store.currentTaskId).toBeNull();
  });

  it("deleteTask 删除任务", () => {
    const store = useProjectStore();
    const task1: Task = {
      id: "t1", name: "任务1", task_type: "detection",
      image_folder: "/img1", created_at: "", updated_at: "",
      classes: [{ id: 0, name: "a", color: "#f00" }],
      stats: { total_images: 1, annotated_images: 0, total_annotations: 0 },
    };
    const task2: Task = {
      id: "t2", name: "任务2", task_type: "detection",
      image_folder: "/img2", created_at: "", updated_at: "",
      classes: [{ id: 0, name: "a", color: "#f00" }],
      stats: { total_images: 1, annotated_images: 0, total_annotations: 0 },
    };
    store.tasks = [task1, task2];
    store.currentTaskId = "t1";

    store.deleteTask("t1");
    expect(store.tasks).toHaveLength(1);
    expect(store.tasks[0].id).toBe("t2");
    expect(store.currentTaskId).toBeNull();
  });

  it("addExportHistory 导出记录 (上限50)", () => {
    const store = useProjectStore();
    const entry = { taskName: "任务X", format: "yolo", time: "2024-01-01T00:00:00Z", path: "/out/yolo" };
    store.addExportHistory(entry);

    expect(store.exportHistory).toHaveLength(1);
    expect(store.exportHistory[0].taskName).toBe("任务X");

    // 测试超过50条截断
    for (let i = 0; i < 60; i++) {
      store.addExportHistory({ taskName: `任务${i}`, format: "yolo", time: "", path: "" });
    }
    expect(store.exportHistory.length).toBeLessThanOrEqual(50);
  });

  it("updateTaskStats 更新统计", () => {
    const store = useProjectStore();
    const task: Task = {
      id: "t1", name: "任务", task_type: "detection",
      image_folder: "/img", created_at: "", updated_at: "",
      classes: [], stats: { total_images: 10, annotated_images: 0, total_annotations: 0 },
    };
    store.tasks = [task];

    store.updateTaskStats("t1", { annotated_images: 5, total_annotations: 8 });
    expect(store.tasks[0].stats.annotated_images).toBe(5);
    expect(store.tasks[0].stats.total_annotations).toBe(8);
  });
});
