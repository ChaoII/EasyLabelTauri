import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  TOOL_LABELS,
  type Annotation,
  type ClassDefinition,
  type LoadImageResult,
  type ToolName,
  type Point,
  type PolygonAnnotation,
  type RotatedBox,
  type AxisAlignedBox,
  type ImageInfo,
  type ClassificationAnnotation,
  type KeypointAnnotation,
  type Keypoint,
  type OcrAnnotation,
  type Visibility,
} from "@/utils/types";

export const useAppStore = defineStore("app", () => {
  // ==================== 状态 ====================
  const imageLoaded = ref(false);
  const imageBase64 = ref<string | null>(null);
  const imageWidth = ref(0);
  const imageHeight = ref(0);
  const imagePath = ref("");
  const annotations = ref<Annotation[]>([]);
  const classes = ref<ClassDefinition[]>([]);
  const activeTool = ref<ToolName>("select");
  const selectedAnnotationId = ref<string | null>(null);
  const zoom = ref(1);
  const panX = ref(0);
  const panY = ref(0);
  const statusMessage = ref("就绪");
  const cursorX = ref(0);
  const cursorY = ref(0);

  // 画布尺寸（画布实际像素宽高）
  const canvasWidth = ref(0);
  const canvasHeight = ref(0);

  // 绘图中的临时数据
  const drawingPoints = ref<Point[]>([]);
  const drawingStart = ref<Point | null>(null);

  // 关键点绘制状态
  // 阶段1: 放置角点 (kp_phase = 'corners')
  // 阶段2: 绘制包围框 (kp_phase = 'box')
  const kpPhase = ref<"corners" | "box" | null>(null);
  const kpCorners = ref<{ x: number; y: number; name: string; visibility: 0 | 1 | 2 }[]>([]);
  const kpNames = ref<string[]>([]);
  // 阶段2: 包围框预览（跟随鼠标）
  const kpBoxPreview = ref<{ x1: number; y1: number; x2: number; y2: number } | null>(null);

  // OCR 绘制状态
  const ocrDrawingPoints = ref<Point[]>([]);
  const ocrText = ref("");
  const ocrRectMode = ref(false); // false=四边形(点击放置), true=矩形(拖拽)

  // ==================== 图片夹/列表 ====================
  const folderPath = ref("");
  const images = ref<ImageInfo[]>([]);
  const currentImageIndex = ref(-1);
  const imageAnnotationMap = ref<Record<string, boolean>>({});

  const currentImage = computed(() => {
    if (currentImageIndex.value >= 0 && currentImageIndex.value < images.value.length) {
      return images.value[currentImageIndex.value];
    }
    return null;
  });

  const hasPrevImage = computed(() => currentImageIndex.value > 0);
  const hasNextImage = computed(() => currentImageIndex.value < images.value.length - 1);
  const imageCounter = computed(() => {
    if (images.value.length === 0) return "无图片";
    return `${currentImageIndex.value + 1} / ${images.value.length}`;
  });

  // ==================== 计算属性 ====================
  const selectedAnnotation = computed(() =>
    annotations.value.find((a) => a.id === selectedAnnotationId.value) ?? null
  );

  const annotationCount = computed(() => annotations.value.length);

  // ==================== 内部：保存当前图片标注 ====================
  async function saveCurrentAnnotations() {
    if (!imagePath.value) return;
    try {
      await invoke("save_annotations_for_image", {
        imagePath: imagePath.value,
        annotations: annotations.value,
      });
    } catch (e) {
      console.error("保存标注失败:", e);
    }
    // 同步更新任务统计
    try {
      const { useProjectStore } = await import("./project");
      const projectStore = useProjectStore();
      if (projectStore.currentTaskId) {
        const annotated = Object.values(imageAnnotationMap.value).filter(Boolean).length;
        const task = projectStore.tasks.find(t => t.id === projectStore.currentTaskId);
        if (task) {
          task.stats.annotated_images = annotated;
          projectStore.saveProject();
        }
      }
    } catch {}
  }

  // ==================== 内部：加载图片并读取其标注 ====================
  async function loadImageFromPath(path: string) {
    try {
      const result: LoadImageResult = await invoke("load_image", { path });
      imageBase64.value = result.base64;
      imageWidth.value = result.width;
      imageHeight.value = result.height;
      imagePath.value = result.image_path;
      imageLoaded.value = true;
      selectedAnnotationId.value = null;
      zoom.value = 1;
      panX.value = 0;
      panY.value = 0;
      drawingPoints.value = [];
      drawingStart.value = null;
      resetKeypointDraw();
      resetOcrDraw();
      await refreshClasses();

      // 尝试读取该图片的标注文件
      try {
        const savedAnnotations = await invoke<Annotation[]>("load_annotations_for_image", {
          imagePath: path,
        });
        annotations.value = savedAnnotations;
        statusMessage.value = `已加载: ${result.width} × ${result.height}，标注: ${savedAnnotations.length} 个`;
      } catch {
        annotations.value = [];
        statusMessage.value = `已加载: ${result.width} × ${result.height}`;
      }
    } catch (e) {
      statusMessage.value = `错误: ${e}`;
      console.error(e);
    }
  }

  // ==================== 图片操作 ====================
  async function loadImage() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "图像文件", extensions: ["png", "jpg", "jpeg", "bmp", "gif", "webp", "tiff", "tif"] }],
      });
      if (!selected) {
        statusMessage.value = "已取消打开";
        return;
      }
      // 清空文件夹模式，回到单图模式
      folderPath.value = "";
      images.value = [];
      currentImageIndex.value = -1;
      await loadImageFromPath(selected);
    } catch (e) {
      statusMessage.value = `错误: ${e}`;
      console.error(e);
    }
  }

  async function loadFolder() {
    try {
      const selected = await open({ directory: true, title: "选择图片文件夹" });
      if (!selected) {
        statusMessage.value = "已取消打开文件夹";
        return;
      }
      const folderInfo = await invoke<{ folder_path: string; images: ImageInfo[] }>("load_folder", {
        folderPath: selected,
      });
      if (folderInfo.images.length === 0) {
        statusMessage.value = "文件夹中没有图片";
        return;
      }
      folderPath.value = folderInfo.folder_path;
      images.value = folderInfo.images;

      // 检测每张图是否有标注文件
      const map: Record<string, boolean> = {};
      for (const img of folderInfo.images) {
        try {
          const anns = await invoke<Annotation[]>("load_annotations_for_image", {
            imagePath: img.path,
          });
          map[img.path] = anns.length > 0;
        } catch {
          map[img.path] = false;
        }
      }
      imageAnnotationMap.value = map;
      currentImageIndex.value = 0;
      await loadImageFromPath(folderInfo.images[0].path);
    } catch (e) {
      statusMessage.value = `错误: ${e}`;
      console.error(e);
    }
  }

  async function goToImage(index: number) {
    if (index < 0 || index >= images.value.length) return;
    await saveCurrentAnnotations();
    currentImageIndex.value = index;
    await loadImageFromPath(images.value[index].path);
    // 更新该图标注状态
    imageAnnotationMap.value[images.value[index].path] = annotations.value.length > 0;
  }

  async function goToNextImage() {
    if (hasNextImage.value) {
      await goToImage(currentImageIndex.value + 1);
    }
  }

  async function goToPrevImage() {
    if (hasPrevImage.value) {
      await goToImage(currentImageIndex.value - 1);
    }
  }

  async function refreshClasses() {
    try {
      classes.value = await invoke<ClassDefinition[]>("get_default_classes");
    } catch {
      // ignore
    }
  }

  // ==================== 项目操作 ====================
  async function saveProject() {
    if (!imageLoaded.value) return;
    await saveCurrentAnnotations();
    statusMessage.value = `标注已保存到图片同目录: ${imagePath.value}.annotations.json`;
  }

  async function exportYolo() {
    if (!imageLoaded.value) return;
    try {
      const dir = await open({ directory: true, title: "选择导出目录" });
      if (!dir) {
        statusMessage.value = "已取消导出";
        return;
      }
      // 保存当前标注
      await saveCurrentAnnotations();
      await invoke("export_yolo", {
        outputDir: dir,
        project: {
          name: "Project",
          image_path: imagePath.value,
          image_width: imageWidth.value,
          image_height: imageHeight.value,
          annotations: annotations.value,
          classes: classes.value,
          classification_labels: [],
        },
      });
      statusMessage.value = "YOLO 格式已导出";
    } catch (e) {
      statusMessage.value = `导出失败: ${e}`;
    }
  }

  // ==================== 标注操作 ====================
  function addAnnotation(annotation: Annotation) {
    annotations.value.push(annotation);
    if (imagePath.value) {
      imageAnnotationMap.value[imagePath.value] = true;
    }
    saveCurrentAnnotations();
  }

  function removeAnnotation(id: string) {
    annotations.value = annotations.value.filter((a) => a.id !== id);
    if (selectedAnnotationId.value === id) {
      selectedAnnotationId.value = null;
    }
    if (imagePath.value) {
      imageAnnotationMap.value[imagePath.value] = annotations.value.length > 0;
    }
    saveCurrentAnnotations();
  }

  function getClassificationIds(): number[] {
    const cls = annotations.value.find((a) => a.type === "Classification");
    return cls ? (cls as ClassificationAnnotation).class_ids : [];
  }

  function setClassification(classId: number) {
    const ids = getClassificationIds();
    let clsAnn: ClassificationAnnotation | undefined =
      annotations.value.find((a) => a.type === "Classification") as
        | ClassificationAnnotation
        | undefined;

    // 如果已选中同一标签，取消选中
    if (ids.length === 1 && ids[0] === classId) {
      if (clsAnn) {
        removeAnnotation(clsAnn.id);
      }
    } else {
      // 替换为选中的标签
      if (clsAnn) {
        clsAnn.class_ids = [classId];
      } else {
        const newAnn: ClassificationAnnotation = {
          id: crypto.randomUUID(),
          type: "Classification",
          class_ids: [classId],
        };
        addAnnotation(newAnn);
      }
    }

    if (imagePath.value) {
      imageAnnotationMap.value[imagePath.value] = true;
    }
  }

  function toggleClassification(classId: number) {
    const ids = getClassificationIds();
    const idx = ids.indexOf(classId);
    let clsAnn: ClassificationAnnotation | undefined =
      annotations.value.find((a) => a.type === "Classification") as
        | ClassificationAnnotation
        | undefined;

    if (idx !== -1) {
      ids.splice(idx, 1);
    } else {
      ids.push(classId);
    }

    if (clsAnn) {
      if (ids.length === 0) {
        removeAnnotation(clsAnn.id);
      } else {
        clsAnn.class_ids = ids;
      }
    } else if (ids.length > 0) {
      const newAnn: ClassificationAnnotation = {
        id: crypto.randomUUID(),
        type: "Classification",
        class_ids: ids,
      };
      addAnnotation(newAnn);
    }

    if (imagePath.value) {
      imageAnnotationMap.value[imagePath.value] = true;
    }
  }

  function selectAnnotation(id: string | null) {
    selectedAnnotationId.value = id;
  }

  function updateAnnotation(id: string, updated: Partial<Annotation>) {
    const idx = annotations.value.findIndex((a) => a.id === id);
    if (idx !== -1) {
      annotations.value[idx] = { ...annotations.value[idx], ...updated } as any;
      saveCurrentAnnotations();
    }
  }

  function clearAnnotations() {
    annotations.value = [];
    selectedAnnotationId.value = null;
  }

  function clearImage() {
    imageBase64.value = null;
    imagePath.value = "";
    imageWidth.value = 0;
    imageHeight.value = 0;
    imageLoaded.value = false;
    selectedAnnotationId.value = null;
    zoom.value = 1;
    panX.value = 0;
    panY.value = 0;
    clearAnnotations();
    drawingPoints.value = [];
    drawingStart.value = null;
    resetKeypointDraw();
    resetOcrDraw();
    statusMessage.value = "就绪";
  }

  // ==================== 工具操作 ====================
  function setTool(tool: ToolName) {
    activeTool.value = tool;
    drawingPoints.value = [];
    drawingStart.value = null;
    resetOcrDraw();
    statusMessage.value = `工具: ${TOOL_LABELS[tool]}`;
  }

  // ==================== 视图操作 ====================
  function setZoom(z: number) {
    zoom.value = Math.max(0.1, Math.min(4, z));
  }

  function setPan(x: number, y: number) {
    panX.value = x;
    panY.value = y;
  }

  function setCursor(x: number, y: number) {
    cursorX.value = Math.round(x);
    cursorY.value = Math.round(y);
  }

  function setCanvasSize(w: number, h: number) {
    canvasWidth.value = w;
    canvasHeight.value = h;
  }

  // ==================== 绘图操作 ====================
  function startDraw(x: number, y: number) {
    drawingStart.value = { x, y };
    if (activeTool.value === "polygon") {
      drawingPoints.value = [{ x, y }];
    }
  }

  // 实时预览
  function updateDraw(x: number, y: number) {
    if (drawingStart.value && (activeTool.value === "rotated_box" || activeTool.value === "box")) {
      drawingStart.value = { x, y };
    }
  }

  function addDrawPoint(x: number, y: number) {
    if (activeTool.value === "polygon") {
      drawingPoints.value.push({ x, y });
    }
  }

  async function finishDraw(
    classId: number,
    startX: number,
    startY: number,
    endX: number,
    endY: number,
    opts?: {
      type?: "box" | "rotated_box" | "polygon";
      cx?: number;
      cy?: number;
      width?: number;
      height?: number;
      angle?: number;
    }
  ) {
    const type = opts?.type ?? activeTool.value;
    if (type === "box") {
      const ann: AxisAlignedBox = await invoke("create_box", {
        classId,
        x1: startX,
        y1: startY,
        x2: endX,
        y2: endY,
      });
      addAnnotation(ann);
    } else if (type === "rotated_box") {
      const ann: RotatedBox = await invoke("create_rotated_box2", {
        classId,
        cx: opts!.cx,
        cy: opts!.cy,
        width: opts!.width,
        height: opts!.height,
        angle: opts!.angle,
      });
      addAnnotation(ann);
    } else if (type === "polygon" && drawingPoints.value.length >= 3) {
      const ann = await invoke<PolygonAnnotation | null>("create_polygon", {
        classId,
        points: drawingPoints.value,
      });
      if (ann) addAnnotation(ann);
      drawingPoints.value = [];
    }
  }

  // ==================== 关键点绘制（角点 + 包围框）====================
  /**
   * 开始关键点绘制流程
   * - 阶段1: 依次放置各个角点
   * - 阶段2: 绘制包围框（由角点自动派生，拖拽调整）
   */
  function startKeypointDraw(classId: number) {
    const cls = classes.value[classId];
    if (cls?.keypoint_names && cls.keypoint_names.length > 0) {
      kpNames.value = cls.keypoint_names;
    } else {
      // 默认车牌 4 个角: top_left, top_right, bottom_right, bottom_left
      kpNames.value = ["top_left", "top_right", "bottom_right", "bottom_left"];
    }
    // 切换类别时清除已有角点（避免残留上一类的关键点）
    kpCorners.value = [];
    kpBoxPreview.value = null;
    kpPhase.value = "corners";
    statusMessage.value = `关键点: 点击放置 "${kpNames.value[0] || ""}" (1/${kpNames.value.length})`;
  }

  /**
   * 放置一个角点（按名称顺序）
   */
  function addKpCorner(x: number, y: number, visibility: 0 | 1 | 2 = 2) {
    if (kpPhase.value !== "corners") return;
    const nextIndex = kpCorners.value.length;
    if (nextIndex >= kpNames.value.length) return;
    kpCorners.value.push({
      x,
      y,
      name: kpNames.value[nextIndex],
      visibility,
    });
    const placed = kpCorners.value.length;
    const total = kpNames.value.length;
    if (placed < total) {
      statusMessage.value = `关键点: 已放置 ${placed}/${total}，放置 "${kpNames.value[placed] || ""}"`;
    } else {
      // 全部角点放置完毕 → 进入矩形框阶段
      kpPhase.value = "box";
      statusMessage.value = `关键点: 角点完成！拖拽绘制包围框 (按 Esc 重新放置角点)`;
    }
  }

  /**
   * 更新矩形框预览（跟随鼠标）
   */
  function updateKpBoxPreview(x1: number, y1: number, x2: number, y2: number) {
    if (kpPhase.value !== "box") return;
    kpBoxPreview.value = { x1, y1, x2, y2 };
  }

  /**
   * 完成关键点标注（角点 + 包围框），在同一条标注记录里
   */
  async function finishKeypointWithBox(classId: number, bboxX1: number, bboxY1: number, bboxX2: number, bboxY2: number) {
    if (kpCorners.value.length === 0) return;
    const corners = kpCorners.value;
    const keypoints: Keypoint[] = corners.map((c) => ({
      x: c.x,
      y: c.y,
      visibility: c.visibility === 0 ? "Hidden" : c.visibility === 1 ? "Occluded" : "Visible",
      name: c.name,
    }));
    try {
      const ann: KeypointAnnotation = await invoke("create_keypoint_with_bbox", {
        classId,
        keypoints,
        bboxX1,
        bboxY1,
        bboxX2,
        bboxY2,
      });
      addAnnotation(ann);
      console.log("[DEBUG finishKW] annotation added, id=", ann.id, "annotations count now", annotations.value.length);
      // 保存完成后，回到 corners 阶段继续画下一个对象
      kpCorners.value = [];
      kpBoxPreview.value = null;
      kpPhase.value = "corners";
      console.log("[DEBUG finishKW] phase reset to corners, kpPhase=", kpPhase.value);
      const nextName = kpNames.value[0] || "";
      statusMessage.value = `关键点: 已保存! 继续放置 "${nextName}" (1/${kpNames.value.length})`;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      statusMessage.value = `关键点: 保存失败 — ${msg}`;
      console.error("finishKeypointWithBox", e);
    }
  }

  /**
   * 完成关键点标注（角点 + 包围框）
   */
  async function finishKeypoint(classId: number) {
    if (kpCorners.value.length === 0) {
      statusMessage.value = "关键点: 请先放置角点";
      return;
    }
    const corners = kpCorners.value;

    // 计算包围框（从角点推导）
    let x1 = Infinity, y1 = Infinity, x2 = -Infinity, y2 = -Infinity;
    for (const c of corners) {
      x1 = Math.min(x1, c.x);
      y1 = Math.min(y1, c.y);
      x2 = Math.max(x2, c.x);
      y2 = Math.max(y2, c.y);
    }

    await finishKeypointWithBox(classId, x1, y1, x2, y2);
  }

  function resetKeypointDraw() {
    kpCorners.value = [];
    kpNames.value = [];
    kpPhase.value = null;          // 完全结束绘制
    kpBoxPreview.value = null;
    statusMessage.value = "就绪";
  }

  // ==================== OCR 绘制 ====================
  function addOcrPoint(x: number, y: number) {
    if (ocrDrawingPoints.value.length < 4) {
      ocrDrawingPoints.value.push({ x, y });
      const remaining = 4 - ocrDrawingPoints.value.length;
      if (remaining > 0) {
        statusMessage.value = `OCR: 还需要 ${remaining} 个顶点`;
      } else {
        statusMessage.value = "OCR: 输入文本后按 Enter 完成";
      }
    }
  }

  function setOcrText(text: string) {
    ocrText.value = text;
  }

  async function finishOcr(classId: number) {
    if (ocrDrawingPoints.value.length !== 4) {
      store.statusMessage = "OCR: 需要 4 个顶点";
      return;
    }
    if (!ocrText.value.trim()) {
      statusMessage.value = "OCR: 请输入文本内容";
      return;
    }
    const ann: OcrAnnotation = await invoke("create_ocr", {
      classId,
      points: ocrDrawingPoints.value,
      text: ocrText.value.trim(),
    });
    addAnnotation(ann);
    resetOcrDraw();
  }

  function resetOcrDraw() {
    ocrDrawingPoints.value = [];
    ocrText.value = "";
    statusMessage.value = "就绪";
  }

  function cancelDraw() {
    drawingStart.value = null;
    drawingPoints.value = [];
  }

  // ==================== 类别管理 ====================
  const DEFAULT_COLORS = [
    "#FF6B6B", "#4ECDC4", "#FFE66D", "#95E1D3",
    "#F38181", "#AA96DA", "#FCBAD3", "#A8D8EA",
    "#FF9F43", "#78E08F", "#3B3A98", "#D980FA",
  ];
  let _colorIndex = 0;
  function nextColor() {
    return DEFAULT_COLORS[_colorIndex++ % DEFAULT_COLORS.length];
  }

  function addClass(name: string, color?: string, keypointNames?: string[], keypointColors?: string[]) {
    if (!name.trim()) return;
    const id = classes.value.length;
    classes.value.push({
      id,
      name: name.trim(),
      color: color || nextColor(),
      keypoint_names: keypointNames && keypointNames.length > 0 ? keypointNames : undefined,
      keypoint_colors: keypointColors && keypointColors.length > 0 ? keypointColors : undefined,
    });
  }

  function updateClass(id: number, name: string, color: string, keypointNames?: string[], keypointColors?: string[]) {
    const cls = classes.value.find((c) => c.id === id);
    if (cls) {
      cls.name = name.trim();
      cls.color = color;
      if (keypointNames !== undefined) {
        cls.keypoint_names = keypointNames.length > 0 ? keypointNames : undefined;
      }
      if (keypointColors !== undefined) {
        cls.keypoint_colors = keypointColors.length > 0 ? keypointColors : undefined;
      }
    }
  }

  function deleteClass(id: number) {
    const idx = classes.value.findIndex((c) => c.id === id);
    if (idx === -1) return;
    classes.value.splice(idx, 1);
    // 重排 id（保持连续）
    classes.value.forEach((c, i) => { c.id = i; });
  }

  // ==================== 初始化 ====================
  refreshClasses();

  return {
    // state
    imageLoaded,
    imageBase64,
    imageWidth,
    imageHeight,
    imagePath,
    annotations,
    classes,
    activeTool,
    selectedAnnotationId,
    zoom,
    panX,
    panY,
    statusMessage,
    cursorX,
    cursorY,
    canvasWidth,
    canvasHeight,
    drawingPoints,
    drawingStart,
    keypointDrawing: kpCorners,
    currentKeypointNames: kpNames,
    kpPhase,
    kpBoxPreview,
    ocrDrawingPoints,
    ocrText,
    ocrRectMode,
    // folder/navigation
    folderPath,
    images,
    currentImageIndex,
    currentImage,
    hasPrevImage,
    hasNextImage,
    imageCounter,
    imageAnnotationMap,
    // computed
    selectedAnnotation,
    annotationCount,
    // actions
    loadImage,
    loadFolder,
    goToImage,
    goToNextImage,
    goToPrevImage,
    saveProject,
    exportYolo,
    addAnnotation,
    removeAnnotation,
    selectAnnotation,
    updateAnnotation,
    clearAnnotations,
    clearImage,
    setTool,
    setZoom,
    setPan,
    setCursor,
    setCanvasSize,
    startDraw,
    updateDraw,
    addDrawPoint,
    finishDraw,
    cancelDraw,
    startKeypointDraw,
    addKpCorner,
    updateKpBoxPreview,
    finishKeypoint,
    finishKeypointWithBox,
    resetKeypointDraw,
    addOcrPoint,
    setOcrText,
    finishOcr,
    resetOcrDraw,
    addClass,
    updateClass,
    deleteClass,
    getClassificationIds,
    setClassification,
    toggleClassification,
  };
});
