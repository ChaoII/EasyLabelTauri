import { describe, it, expect, beforeEach, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useAppStore } from "../app";
import type { AxisAlignedBox, PolygonAnnotation, KeypointAnnotation, OcrAnnotation } from "@/utils/types";

// 模拟 Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

// 模拟 Tauri dialog
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

describe("useAppStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("初始状态正确", () => {
    const store = useAppStore();
    expect(store.imageLoaded).toBe(false);
    expect(store.annotations).toEqual([]);
    expect(store.activeTool).toBe("select");
    expect(store.selectedAnnotationId).toBeNull();
    expect(store.history).toEqual([]);
    expect(store.historyIndex).toBe(-1);
  });

  it("addAnnotation 写入标注并触发历史", () => {
    const store = useAppStore();
    const ann: AxisAlignedBox = {
      id: "ann-1", type: "AxisAlignedBox", class_id: 0,
      x1: 0.1, y1: 0.1, x2: 0.5, y2: 0.5, locked: false,
    };
    store.addAnnotation(ann);
    expect(store.annotations).toHaveLength(1);
    expect(store.annotations[0].id).toBe("ann-1");
    expect(store.historyIndex).toBe(0);
  });

  it("removeAnnotation 删除指定标注", () => {
    const store = useAppStore();
    store.addAnnotation({ id: "a1", type: "AxisAlignedBox", class_id: 0, x1: 0, y1: 0, x2: 0.5, y2: 0.5, locked: false });
    store.addAnnotation({ id: "a2", type: "AxisAlignedBox", class_id: 1, x1: 0.1, y1: 0.1, x2: 0.6, y2: 0.6, locked: false });
    store.removeAnnotation("a1");
    expect(store.annotations).toHaveLength(1);
    expect(store.annotations[0].id).toBe("a2");
  });

  it("updateAnnotation 更新指定字段", () => {
    const store = useAppStore();
    store.addAnnotation({ id: "a1", type: "AxisAlignedBox", class_id: 0, x1: 0, y1: 0, x2: 0.5, y2: 0.5, locked: false });
    store.updateAnnotation("a1", { class_id: 2 });
    expect(store.annotations[0].class_id).toBe(2);
  });

  it("pushHistory / undo / redo 正常", () => {
    const store = useAppStore();
    // addAnnotation 内部会调用 pushHistory
    store.addAnnotation({
      id: "a1", type: "AxisAlignedBox", class_id: 0,
      x1: 0, y1: 0, x2: 0.5, y2: 0.5, locked: false,
    });
    expect(store.historyIndex).toBe(0);
    expect(store.annotations).toHaveLength(1);

    store.addAnnotation({
      id: "a2", type: "AxisAlignedBox", class_id: 1,
      x1: 0.1, y1: 0.1, x2: 0.6, y2: 0.6, locked: false,
    });
    expect(store.historyIndex).toBe(1);
    expect(store.annotations).toHaveLength(2);

    store.undo();
    expect(store.annotations).toHaveLength(1);
    expect(store.annotations[0].id).toBe("a1");
    expect(store.historyIndex).toBe(0);

    store.redo();
    expect(store.annotations).toHaveLength(2);
    expect(store.historyIndex).toBe(1);
  });

  it("copySelectedAnnotation / pasteAnnotation 复制粘贴", () => {
    const store = useAppStore();
    store.imagePath = "test.jpg";

    store.addAnnotation({ id: "a1", type: "AxisAlignedBox", class_id: 0, x1: 0.1, y1: 0.2, x2: 0.5, y2: 0.6, locked: false });
    expect(store.annotations).toHaveLength(1);

    // 复制前要先选中
    store.selectedAnnotationId = "a1";
    store.copySelectedAnnotation();
    expect(store.copiedAnnotation).not.toBeNull();
    expect(store.copiedAnnotation!.id).toBe("a1");

    store.pasteAnnotation();
    expect(store.annotations).toHaveLength(2);
    // 粘贴后的标注 id 应该是新的
    expect(store.annotations[1].id).not.toBe("a1");
    // 坐标偏移 0.01
    const pasted = store.annotations[1] as AxisAlignedBox;
    expect(pasted.x1).toBeCloseTo(0.11);
    expect(pasted.y2).toBeCloseTo(0.61);
  });

  it("toggleLock 切换锁定状态", () => {
    const store = useAppStore();
    store.addAnnotation({ id: "a1", type: "AxisAlignedBox", class_id: 0, x1: 0, y1: 0, x2: 0.5, y2: 0.5, locked: false });
    expect((store.annotations[0] as any).locked).toBe(false);

    store.toggleLock("a1");
    expect((store.annotations[0] as any).locked).toBe(true);

    store.toggleLock("a1");
    expect((store.annotations[0] as any).locked).toBe(false);
  });

  it("batchDeleteSelected 批量删除", () => {
    const store = useAppStore();
    store.addAnnotation({ id: "a1", type: "AxisAlignedBox", class_id: 0, x1: 0, y1: 0, x2: 0.5, y2: 0.5, locked: false });
    store.addAnnotation({ id: "a2", type: "AxisAlignedBox", class_id: 1, x1: 0.1, y1: 0.1, x2: 0.6, y2: 0.6, locked: false });
    store.addAnnotation({ id: "a3", type: "AxisAlignedBox", class_id: 2, x1: 0.2, y1: 0.2, x2: 0.7, y2: 0.7, locked: false });

    store.toggleBatchSelect("a1");
    store.toggleBatchSelect("a3");
    expect(store.selectedAnnotationIds.size).toBe(2);

    store.batchDeleteSelected();
    expect(store.annotations).toHaveLength(1);
    expect(store.annotations[0].id).toBe("a2");
    expect(store.selectedAnnotationIds.size).toBe(0);
  });

  it("batchUpdateClassSelected 批量修改类别", () => {
    const store = useAppStore();
    store.addAnnotation({ id: "a1", type: "AxisAlignedBox", class_id: 0, x1: 0, y1: 0, x2: 0.5, y2: 0.5, locked: false });
    store.addAnnotation({ id: "a2", type: "AxisAlignedBox", class_id: 1, x1: 0.1, y1: 0.1, x2: 0.6, y2: 0.6, locked: false });

    store.toggleBatchSelect("a1");
    store.toggleBatchSelect("a2");
    store.batchUpdateClassSelected(3);

    expect(store.annotations[0].class_id).toBe(3);
    expect(store.annotations[1].class_id).toBe(3);
  });

  it("setTool 切换工具", () => {
    const store = useAppStore();
    store.setTool("box");
    expect(store.activeTool).toBe("box");
    store.setTool("polygon");
    expect(store.activeTool).toBe("polygon");
    store.setTool("ocr");
    expect(store.activeTool).toBe("ocr");
  });

  it("多边形绘制: addDrawPoint / cancelDraw / finishDraw", () => {
    const store = useAppStore();
    store.activeTool = "polygon";
    store.imagePath = "test.jpg";
    store.imageLoaded = true;

    store.addDrawPoint(0.1, 0.1);
    store.addDrawPoint(0.3, 0.1);
    store.addDrawPoint(0.3, 0.3);
    expect(store.drawingPoints).toHaveLength(3);

    store.cancelDraw();
    expect(store.drawingPoints).toHaveLength(0);
  });

  it("selectAnnotation 选中", () => {
    const store = useAppStore();
    store.addAnnotation({ id: "a1", type: "AxisAlignedBox", class_id: 0, x1: 0, y1: 0, x2: 0.5, y2: 0.5, locked: false });
    store.selectAnnotation("a1");
    expect(store.selectedAnnotationId).toBe("a1");
    expect(store.selectedAnnotation?.id).toBe("a1");

    store.selectAnnotation(null);
    expect(store.selectedAnnotationId).toBeNull();
  });

  it("setZoom / setZoomRange 范围限制", () => {
    const store = useAppStore();
    store.setZoom(2);
    expect(store.zoom).toBe(2);
    store.setZoom(0.05);
    expect(store.zoom).toBe(0.1);
    store.setZoom(5);
    expect(store.zoom).toBe(4);
  });
});
