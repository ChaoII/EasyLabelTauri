<template>
  <div class="ann-page">
    <!-- 统一顶部栏 -->
    <AppHeader>
      <template #center>
        <!-- 返回 + 任务信息 -->
        <div class="ann-title">
          <button class="back-btn" @click="emit('back')" title="返回任务列表">
            <ChevronLeft :size="16" />
          </button>
          <div class="task-type-badge" :style="{ background: typeColor + '22', color: typeColor }">
            {{ TASK_TYPE_ICONS[task?.task_type ?? 'detection'] }}
          </div>
          <span class="task-name">{{ task?.name }}</span>
        </div>

        <!-- 顶栏内进度条 -->
        <div class="header-progress" v-if="totalCount > 0">
          <span class="progress-text">{{ annotatedCount }} / {{ totalCount }}</span>
          <div class="progress-mini">
            <div class="progress-mini-fill" :class="{ 'fill-done': progressPct >= 100 }" :style="{ width: progressPct + '%' }" />
          </div>
        </div>
      </template>
    </AppHeader>

    <!-- 主体：左侧工具 + 画布 + 右侧面板 -->
    <div class="ann-body">
      <!-- 左侧工具栏 -->
      <aside class="ann-leftbar">
        <div class="tool-list">
          <!-- 基础工具 -->
          <ToolButton
            v-for="tool in baseTools"
            :key="tool.name"
            v-model="activeTool"
            :tool="tool"
          >
            <component :is="tool.icon" />
          </ToolButton>

          <div class="tool-sep" />

          <!-- 任务专属工具 -->
          <ToolButton
            v-for="tool in taskTools"
            :key="tool.name"
            v-model="activeTool"
            :tool="tool"
          >
            <component :is="tool.icon" />
          </ToolButton>

          <template v-if="task?.task_type === 'classification'">
            <div class="tool-sep" />
            <ToolButton
              v-for="tool in clsTools"
              :key="tool.name"
              v-model="activeTool"
              :tool="tool"
            >
              <component :is="tool.icon" />
            </ToolButton>
          </template>
        </div>
      </aside>

      <!-- 中间画布 -->
      <main class="ann-canvas-area">
        <Canvas
          :selected-class-id="selectedClassId"
          @open-image="loadCurrentImage"
        />
      </main>

      <!-- 右侧面板 -->
      <aside class="ann-rightbar">
        <div class="panel">

          <!-- 图片列表 -->
          <div class="panel-section">
            <div class="section-title-row">
              <span class="section-title">图片列表</span>
              <span v-if="taskImages.length > 0" class="count-chip">{{ taskImages.length }}</span>
              <button class="icon-btn" title="重新加载目录" @click="reloadFolder">
                <RefreshCw :size="13" />
              </button>
            </div>
            <div class="image-list" ref="imageListRef" @scroll="onImageListScroll" v-if="taskImages.length > 0">
              <div
                v-for="(img, idx) in taskImages"
                :key="img.path"
                class="image-item"
                :class="{ active: idx === currentImageIndex }"
                @click="goToImage(idx)"
              >
                <span
                  class="ann-dot"
                  :class="(imageAnnotationMap[img.path] ?? false) ? 'dot-done' : 'dot-pending'"
                />
                <span class="img-name">{{ img.name }}</span>
              </div>
            </div>
            <div v-else class="panel-empty">暂无图片</div>
          </div>

          <div class="panel-divider" />

          <!-- 标签类别 -->
          <div class="panel-section">
            <div class="section-title-row">
              <span class="section-title">标签类别</span>
              <button class="icon-btn" title="添加类别" @click="classModalRef?.openAsAdd()">
                <Plus :size="13" />
              </button>
            </div>
            <div class="section-scroll">
              <ClassList
                :classes="task?.classes ?? []"
                :selected-class-id="selectedClassId"
                :annotations="store.annotations"
                :multi-mode="task?.task_type === 'classification'"
                :selected-class-ids="store.getClassificationIds()"
                @select="(id) => selectedClassId = id"
                @toggle="(id) => store.toggleClassification(id)"
                @edit="(cls) => classModalRef?.openAsEdit(cls)"
              />
            </div>
          </div>

          <div class="panel-divider" />

          <!-- 标注列表 -->
          <div class="panel-section">
            <div class="section-title-row">
              <span class="section-title">标注列表</span>
              <NBadge :value="store.annotationCount" :max="999" type="warning" />
            </div>
            <div class="section-scroll">
              <AnnotationList
                :annotations="store.annotations"
                :selected-id="store.selectedAnnotationId"
                :classes="task?.classes ?? []"
                @select="(id) => store.selectAnnotation(id)"
                @delete="(id) => store.removeAnnotation(id)"
              />
            </div>
          </div>

        </div>
      </aside>
    </div>

    <!-- 底栏 -->
    <footer class="ann-statusbar" v-show="settingsStore.settings.status_bar_visible">
      <span class="status-text">{{ store.statusMessage }}</span>
      <span class="hint">{{ toolHint }}</span>
      <span class="coords">X: {{ store.cursorX }}  Y: {{ store.cursorY }}</span>

      <template v-if="taskImages.length > 1">
        <div class="nav-sep" />
        <button class="nav-btn" @click="goToPrevImage()" :disabled="currentImageIndex <= 0" title="上一张">
          <ChevronLeft :size="14" />
        </button>
        <span class="nav-counter">{{ currentImageIndex + 1 }} / {{ taskImages.length }}</span>
        <button class="nav-btn" @click="goToNextImage()" :disabled="currentImageIndex >= taskImages.length - 1" title="下一张">
          <ChevronRight :size="14" />
        </button>
      </template>

      <div class="nav-sep" />

      <button class="save-btn" @click="saveCurrentAnnotations()" :disabled="!store.imageLoaded">
        保存
      </button>
    </footer>

    <ClassModal
      ref="classModalRef"
      @add="(name, color, kpnames) => { store.addClass(name, color, kpnames); syncClassesToProject(); }"
      @update="(id, name, color, kpnames) => { store.updateClass(id, name, color, kpnames); syncClassesToProject(); }"
      @delete="(id) => { store.deleteClass(id); syncClassesToProject(); }"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { NBadge } from "naive-ui";
import {
  MousePointer2, Square, Pentagon, CircleDot, Type,
  Hand, ZoomIn, ChevronLeft, ChevronRight, RefreshCw,
  AlignLeft, Diamond, Plus,
} from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import { useProjectStore } from "@/stores/project";
import { useSettingsStore } from "@/stores/settings";
import { TASK_TYPE_ICONS, type TaskType } from "@/utils/taskTypes";
import ToolButton from "./ToolButton.vue";
import Canvas from "./Canvas.vue";
import ClassList from "./ClassList.vue";
import AnnotationList from "./AnnotationList.vue";
import ClassModal from "./ClassModal.vue";
import AppHeader from "./AppHeader.vue";
import type { ToolName, ImageInfo } from "@/utils/types";
import type { Component } from "vue";

const props = defineProps<{ taskId: string }>();
const emit = defineEmits<{ (e: "back"): void }>();

const store = useAppStore();
const projectStore = useProjectStore();
const settingsStore = useSettingsStore();

const task = computed(() => projectStore.tasks.find((t) => t.id === props.taskId));
const taskImages = ref<ImageInfo[]>([]);
const currentImageIndex = ref(0);
// eslint-disable-next-line
const imageAnnotationMap = ref({} as Record<string, boolean>);
const selectedClassId = ref(0);
watch(() => store.classes.length, (len) => {
  if (len > 0) {
    if (selectedClassId.value >= len) selectedClassId.value = 0;
  }
});
const classModalRef = ref<InstanceType<typeof ClassModal> | null>(null);
const imageListRef = ref<HTMLElement | null>(null);

// ==================== 工具栏 ====================
const activeTool = ref<ToolName>("select");
watch(activeTool, (t) => store.setTool(t), { immediate: true });

const baseTools = [
  { name: "select" as ToolName, label: "选择", tip: "选择并编辑标注", icon: MousePointer2 },
  { name: "pan" as ToolName, label: "平移", tip: "拖动画布平移（滚轮缩放）", icon: Hand },
  { name: "zoom" as ToolName, label: "缩放", tip: "滚轮缩放画布", icon: ZoomIn },
];

const taskToolMap: Record<TaskType, { name: ToolName; label: string; tip: string; icon: Component }[]> = {
  detection: [{ name: "box", label: "矩形", tip: "点击并拖拽创建普通矩形框", icon: Square }],
  rotated_detection: [{ name: "rotated_box", label: "旋转框", tip: "三步绘制旋转矩形", icon: Diamond }],
  keypoint: [{ name: "keypoint", label: "关键点", tip: "依次点击放置关键点角点", icon: CircleDot }],
  segmentation: [{ name: "polygon", label: "多边形", tip: "点击添加顶点，单击起始点闭合", icon: Pentagon }],
  ocr: [{ name: "ocr", label: "OCR", tip: "按 T 切换矩形/四边形模式，矩形拖拽绘制，四边形点击放置顶点", icon: Type }],
  classification: [],
};

const clsTools = [
  { name: "classification" as ToolName, label: "分类", tip: "为图像添加类别标签", icon: AlignLeft },
];

const taskTools = computed(() => taskToolMap[task.value?.task_type ?? "detection"] ?? []);

const typeColor = computed(() => {
  const m: Record<TaskType, string> = {
    classification: "#f97316", detection: "#38bdf8", rotated_detection: "#a855f7",
    keypoint: "#fbbf24", segmentation: "#4ade80", ocr: "#eab308",
  };
  return m[task.value?.task_type ?? "detection"] ?? "#6b7280";
});

// ==================== 工具提示 ====================
const toolHint = computed(() => {
  const h: Record<ToolName, string> = {
    select: "点击选择已有标注，拖拽可框选",
    pan: "拖动画布平移视图（滚轮缩放）",
    box: "点击并拖拽创建普通矩形框",
    rotated_box: "三步：点一边 → 拖宽度 → 点定高度",
    polygon: "点击添加顶点，单击起始点闭合多边形",
    keypoint: "依次点击放置关键点角点",
    ocr: "点击放置顶点，输入文本",
    zoom: "滚轮向上放大、向下缩小",
    classification: "为图像添加类别标签",
  };
  return h[activeTool.value] ?? "";
});

// ==================== 生命周期 ====================
onMounted(async () => {
  await loadTaskImages();
  if (taskImages.value.length > 0) {
    await loadImageFromPath(taskImages.value[0].path);
  }
});

// ==================== 图片 ====================
// ==================== 图片目录：按需懒加载标注状态 ====================
// 为每张图单独发起标注查询，查完才更新（不阻塞首图展示）
async function loadAnnotationStatus(idx: number) {
  const img = taskImages.value[idx];
  if (!img || imageAnnotationMap.value[img.path] !== undefined) return; // 已有则跳过
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const anns = await invoke<any[]>("load_annotations_for_image", { imagePath: img.path });
    imageAnnotationMap.value[img.path] = anns.length > 0;
  } catch { imageAnnotationMap.value[img.path] = false; }
}

async function loadTaskImages() {
  if (!task.value) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<{ folder_path: string; images: ImageInfo[] }>("load_folder", {
      folderPath: task.value!.image_folder,
    });
    taskImages.value = result.images;

    // 立即显示第一张图（不等待标注状态）
    if (result.images.length > 0) {
      await loadImageFromPath(result.images[0].path);
    }

    // 标注状态后台按需加载：只查当前图及附近几张，其余等滚动到时再查
    const initialIndices = [0];
    if (result.images.length > 1) initialIndices.push(1);
    if (result.images.length > 2) initialIndices.push(2);

    await Promise.all(initialIndices.map((i) => loadAnnotationStatus(i)));
  } catch (e) {
    console.error("加载图片目录失败:", e);
  }
}

async function loadImageFromPath(path: string) {
  // 先清空旧图，防止切换时旧图残留闪烁
  store.clearImage();
  // 同步任务类别（含 keypoint_names）到 app store
  if (task.value?.classes && task.value.classes.length > 0) {
    store.classes = task.value.classes;
  }
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result: any = await invoke("load_image", { path });
    // 画布用 convertFileSrc(真实路径) 加载；base64 字段可能为空（仅元数据模式）
    store.imagePath = result.image_path ?? path;
    store.imageBase64 = result.base64 ?? null;
    store.imageWidth = result.width;
    store.imageHeight = result.height;
    store.imageLoaded = true;
    store.selectedAnnotationId = null;
    store.zoom = 1;
    store.panX = 0;
    store.panY = 0;
    store.clearAnnotations();
    const saved: any[] = await invoke("load_annotations_for_image", { imagePath: path });
    store.annotations = saved;
  } catch (e) {
    console.error("加载图片失败:", e);
  }
}

function loadCurrentImage() {
  if (taskImages.value.length > 0) {
    loadImageFromPath(taskImages.value[currentImageIndex.value].path);
  }
}

// 图片列表滚动时：懒加载可见区域的标注状态
function onImageListScroll() {
  const el = imageListRef.value;
  if (!el) return;
  const indices = new Set<number>();
  // 遍历所有列表项，判断是否在可视区域内
  const items = el.querySelectorAll<HTMLElement>(".image-item");
  items.forEach((item, idx) => {
    const rect = item.getBoundingClientRect();
    const parentRect = el.getBoundingClientRect();
    if (rect.top >= parentRect.top && rect.bottom <= parentRect.bottom) {
      indices.add(idx);
    }
  });
  // 并行发起查询（已在加载中的会被 skip）
  indices.forEach((idx) => loadAnnotationStatus(idx));
}

function goToImage(idx: number) {
  currentImageIndex.value = idx;
  loadImageFromPath(taskImages.value[idx].path);
}

function goToNextImage() {
  if (currentImageIndex.value < taskImages.value.length - 1) goToImage(currentImageIndex.value + 1);
}

function goToPrevImage() {
  if (currentImageIndex.value > 0) goToImage(currentImageIndex.value - 1);
}

async function reloadFolder() {
  await loadTaskImages();
  currentImageIndex.value = 0;
  if (taskImages.value.length > 0) {
    await loadImageFromPath(taskImages.value[0].path);
  }
}

async function saveCurrentAnnotations() {
  if (!store.imagePath) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("save_annotations_for_image", {
      imagePath: store.imagePath,
      annotations: store.annotations,
    });
    imageAnnotationMap.value[store.imagePath] = store.annotations.length > 0;
    const annotated = Object.values(imageAnnotationMap.value).filter(Boolean).length;
    await projectStore.updateTaskStats(props.taskId, { annotated_images: annotated });
  } catch (e) {
    console.error("保存失败:", e);
  }
}

async function syncClassesToProject() {
  if (props.taskId) {
    await projectStore.updateTaskClasses(props.taskId, store.classes);
  }
}

// ==================== 统计 ====================
const annotatedCount = computed(() =>
  Object.values(imageAnnotationMap.value).filter(Boolean).length
);
const totalCount = computed(() => taskImages.value.length);
const progressPct = computed(() =>
  totalCount.value === 0 ? 0 : Math.round((annotatedCount.value / totalCount.value) * 100)
);
</script>

<style scoped>
/* ==================== 整体布局 ==================== */
.ann-page {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  background: var(--bg-app);
  overflow: hidden;
}

/* ---- 顶栏内中间区域 ---- */
.ann-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
  flex-shrink: 0;
}

.back-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.task-type-badge {
  font-size: 13px;
  font-weight: 700;
  padding: 2px 7px;
  border-radius: 4px;
  line-height: 1;
}

.task-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 240px;
}

.header-progress {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.progress-text {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}

.progress-mini {
  width: 80px;
  height: 4px;
  background: var(--bg-elevated);
  border-radius: 2px;
  overflow: hidden;
}

.progress-mini-fill {
  height: 100%;
  border-radius: 2px;
  background: var(--accent);
  transition: width 0.3s;
}

.progress-mini-fill.fill-done {
  background: #22c55e;
}

/* ---- 主体 ---- */
.ann-body {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

/* ---- 左侧工具栏 ---- */
.ann-leftbar {
  width: var(--leftbar-width);
  background: var(--bg-panel);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  align-items: center;
  flex-shrink: 0;
  padding: 8px 2px 10px;
}

.tool-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  width: 100%;
}

.tool-sep {
  align-self: stretch;
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 2px;
}

/* ---- 画布区域 ---- */
.ann-canvas-area {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* ---- 右侧栏 ---- */
.ann-rightbar {
  width: var(--rightbar-width);
  background: var(--bg-panel);
  border-left: 1px solid var(--border-subtle);
  flex-shrink: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.panel {
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 0;
  flex: 1;
  min-height: 0;
}

.section-scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.panel-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.section-title-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  flex: 1;
}

.count-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-secondary);
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  padding: 0 6px;
  min-height: 18px;
  border-radius: 999px;
  font-variant-numeric: tabular-nums;
}

.panel-divider {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
}

.panel-empty {
  font-size: 11px;
  color: var(--text-dim);
  padding: 8px 0;
  text-align: center;
}

/* ---- 图片列表 ---- */
.image-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  padding: 4px;
  background: var(--bg-elevated);
}

.image-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid transparent;
  border-left: 3px solid transparent;
  transition: background 0.1s;
}

.image-item:hover {
  background: var(--bg-hover);
}

.image-item.active {
  background: rgba(249, 115, 22, 0.12);
  border-color: rgba(249, 115, 22, 0.25);
  border-left-color: var(--accent);
}

.ann-dot {
  flex-shrink: 0;
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.dot-done { background: #16a34a; }
.dot-pending { background: #dc2626; }

.img-name {
  font-size: 11px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ---- 底栏 ---- */
.ann-statusbar {
  height: var(--statusbar-height);
  background: var(--bg-panel);
  border-top: 1px solid var(--border-subtle);
  display: flex;
  align-items: center;
  padding: 0 12px;
  gap: 8px;
  flex-shrink: 0;
}

.status-text {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--text-dim);
  padding-right: 12px;
  margin-right: 4px;
  border-right: 1px solid var(--border-subtle);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hint {
  flex: 1;
  font-size: 11px;
  color: var(--text-dim);
}

.coords {
  font-size: 11px;
  color: var(--text-dim);
  font-family: monospace;
  white-space: nowrap;
}

.nav-sep {
  width: 1px;
  height: 14px;
  background: var(--border-subtle);
  flex-shrink: 0;
}

.nav-counter {
  font-size: 11px;
  color: var(--text-secondary);
  font-family: monospace;
  white-space: nowrap;
  min-width: 60px;
  text-align: center;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
  flex-shrink: 0;
}

.nav-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.save-btn {
  padding: 3px 10px;
  border-radius: 4px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  color: var(--text-primary);
  font-size: 11px;
  cursor: pointer;
  transition: background 0.1s, border-color 0.1s;
  flex-shrink: 0;
  white-space: nowrap;
}

.save-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--accent);
}

.save-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

/* ---- 通用图标按钮 ---- */
.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
  flex-shrink: 0;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
