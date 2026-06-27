<template>
  <div class="ann-page">
    <!-- 统一顶部栏 -->
    <AppHeader>
      <template #center>
        <!-- 返回 + 任务信息 -->
        <div class="ann-title">
          <NButton class="back-btn" @click="emit('back')" title="返回任务列表" quaternary circle size="tiny">
            <template #icon><ChevronLeft :size="16" /></template>
          </NButton>
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
            <ToolButton
              v-for="tool in clsTools"
              :key="tool.name"
              v-model="activeTool"
              :tool="tool"
            >
              <component :is="tool.icon" />
            </ToolButton>
          </template>

          <div class="tool-sep" />
          <NButton quaternary size="tiny" class="ai-btn" @click="openAiAnnotate" title="AI 自动标注">
            <template #icon>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
            </template>
            <span>AI</span>
          </NButton>
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
              <NButton quaternary circle size="tiny" title="重新加载目录" @click="reloadFolder">
                <template #icon><RefreshCw :size="13" /></template>
              </NButton>
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
                  :class="(store.imageAnnotationMap[img.path] ?? false) ? 'dot-done' : 'dot-pending'"
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
              <NButton quaternary circle size="tiny" title="添加类别" @click="classModalRef?.openAsAdd()">
                <template #icon><Plus :size="13" /></template>
              </NButton>
            </div>
            <div class="section-scroll">
              <ClassList
                :classes="task?.classes ?? []"
                :selected-class-id="selectedClassId"
                :annotations="store.annotations"
                :multi-mode="task?.task_type === 'classification'"
                :classification-mode="classificationMode"
                :selected-class-ids="store.getClassificationIds()"
                @select="(id) => selectedClassId = id"
                @toggle="(id) => classificationMode === 'single' ? store.setClassification(id) : store.toggleClassification(id)"
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
                @edit="openAnnEdit"
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
      <span class="shortcut-hint">{{ shortcutHint }}</span>
      <span class="coords">X: {{ store.cursorX }}  Y: {{ store.cursorY }}</span>

      <template v-if="taskImages.length > 1">
        <div class="nav-sep" />
        <NButton quaternary circle size="tiny" @click="goToPrevImage()" :disabled="currentImageIndex <= 0" title="上一张">
          <template #icon><ChevronLeft :size="14" /></template>
        </NButton>
        <span class="nav-counter">{{ currentImageIndex + 1 }} / {{ taskImages.length }}</span>
        <NButton quaternary circle size="tiny" @click="goToNextImage()" :disabled="currentImageIndex >= taskImages.length - 1" title="下一张">
          <template #icon><ChevronRight :size="14" /></template>
        </NButton>
      </template>

      <div class="nav-sep" />

      <NButton size="tiny" type="primary" @click="saveCurrentAnnotations()" :disabled="!store.imageLoaded">
        保存
      </NButton>
    </footer>

    <ClassModal
      ref="classModalRef"
      :task-type="task?.task_type ?? 'detection'"
      @add="(name, color, kpnames, kpcolors) => { store.addClass(name, color, kpnames, kpcolors); syncClassesToProject(); }"
      @update="(id, name, color, kpnames, kpcolors) => { store.updateClass(id, name, color, kpnames, kpcolors); syncClassesToProject(); }"
      @delete="(id) => { store.deleteClass(id); syncClassesToProject(); }"
    />

    <!-- 标注编辑弹窗 -->
    <NModal
      v-model:show="annEditVisible"
      preset="card"
      title="编辑标注"
      :mask-closable="true"
      style="width: 340px"
    >
      <div class="modal-body-edit">
        <div class="field">
          <label class="field-label">类别</label>
          <NSelect
            v-model:value="annEditClassId"
            :options="classOptions"
            size="small"
            placeholder="选择类别"
          />
        </div>
        <div v-if="annEditType === 'Ocr'" class="field">
          <label class="field-label">文本内容</label>
          <NInput
            v-model:value="annEditText"
            size="small"
            placeholder="OCR 文本"
            :maxlength="200"
          />
        </div>
      </div>
      <template #footer>
        <div class="modal-footer">
          <NButton size="small" @click="annEditVisible = false">取消</NButton>
          <NButton size="small" type="primary" @click="confirmAnnEdit">确认</NButton>
        </div>
      </template>
    </NModal>

    <!-- AI 自动标注弹窗 -->
    <NModal v-model:show="showAiAnnotateModal" preset="card" title="AI 自动标注" :mask-closable="true" style="width: 440px">
      <div class="modal-body-export">
        <div class="field">
          <label class="field-label">标注范围</label>
          <div class="mode-switch">
            <NButtonGroup size="small">
              <NButton :type="aiAnnotateMode === 'current' ? 'primary' : 'default'" @click="aiAnnotateMode = 'current'">标注当前图片</NButton>
              <NButton :type="aiAnnotateMode === 'all' ? 'primary' : 'default'" @click="aiAnnotateMode = 'all'">标注所有图片</NButton>
            </NButtonGroup>
          </div>
        </div>
        <div v-if="task?.task_type === 'ocr'" class="field">
          <label class="field-label">检测模型 ONNX</label>
          <div class="dir-row">
            <NInput v-model:value="ocrDetModelPath" size="small" placeholder="选择 det 模型..." />
            <NButton size="small" @click="pickAiModelFile('det')">选择</NButton>
          </div>
          <label class="field-label" style="margin-top:4px">识别模型 ONNX</label>
          <div class="dir-row">
            <NInput v-model:value="ocrRecModelPath" size="small" placeholder="选择 rec 模型..." />
            <NButton size="small" @click="pickAiModelFile('rec')">选择</NButton>
          </div>
          <label class="field-label" style="margin-top:4px">分类模型 ONNX (可选)</label>
          <div class="dir-row">
            <NInput v-model:value="ocrClsModelPath" size="small" placeholder="选择 cls 模型..." />
            <NButton size="small" @click="pickAiModelFile('cls')">选择</NButton>
          </div>
          <label class="field-label" style="margin-top:4px">字典文件</label>
          <div class="dir-row">
            <NInput v-model:value="ocrDictPath" size="small" placeholder="选择 dict.txt..." />
            <NButton size="small" @click="pickAiModelFile('dict')">选择</NButton>
          </div>
        </div>
        <div v-else class="field">
          <label class="field-label">模型 ONNX 路径</label>
          <div class="dir-row">
            <NInput v-model:value="aiModelPath" size="small" placeholder="选择模型文件..." />
            <NButton size="small" @click="pickAiModelFile('model')">选择</NButton>
          </div>
        </div>
        <div v-if="aiAnnotating" class="export-progress">
          <div class="progress-bar-outer">
            <div class="progress-bar-inner" :style="{ width: (aiAnnotateProgress.total > 0 ? aiAnnotateProgress.current / aiAnnotateProgress.total * 100 : 0) + '%' }" />
          </div>
          <span class="progress-text">{{ aiAnnotateProgress.message }} {{ aiAnnotateProgress.total > 0 ? aiAnnotateProgress.current + '/' + aiAnnotateProgress.total : '' }}</span>
        </div>
      </div>
      <template #footer>
        <div class="drawer-footer">
          <NButton size="small" @click="showAiAnnotateModal = false" :disabled="aiAnnotating">取消</NButton>
          <NButton size="small" type="primary" @click="handleAiAnnotate" :loading="aiAnnotating">开始标注</NButton>
        </div>
      </template>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";
import { NBadge, NModal, NSelect, NInput, NButton, useMessage } from "naive-ui";
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
const message = useMessage();

const task = computed(() => projectStore.tasks.find((t) => t.id === props.taskId));
const classificationMode = computed(() => task.value?.config?.classification_mode ?? "multi");
const taskImages = ref<ImageInfo[]>([]);
const currentImageIndex = ref(0);
// eslint-disable-next-line
// 直接使用 store.imageAnnotationMap，无需本地包装
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
    classification: "#6366f1", detection: "#3b82f6", rotated_detection: "#8b5cf6",
    keypoint: "#eab308", segmentation: "#22c55e", ocr: "#06b6d4",
  };
  return m[task.value?.task_type ?? "detection"] ?? "#6b7280";
});

// ==================== 快捷键 ====================
const toolKeyMap: Record<string, ToolName> = {
  "1": "select", "s": "select",
  "2": "box", "b": "box",
  "3": "rotated_box", "r": "rotated_box",
  "4": "polygon", "p": "polygon",
  "5": "keypoint", "k": "keypoint",
  "6": "ocr", "o": "ocr",
  "7": "classification", "c": "classification",
  "h": "pan",
  "z": "zoom",
};

function onKeyDown(e: KeyboardEvent) {
  // 忽略输入框中的按键
  const tag = (e.target as HTMLElement)?.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

  const key = e.key.toLowerCase();

  // Ctrl+S 保存
  if (e.ctrlKey && key === "s") {
    e.preventDefault();
    saveCurrentAnnotations();
    return;
  }

  // Delete / Backspace 删除选中标注
  if (key === "delete" || key === "backspace") {
    if (store.selectedAnnotationId) {
      store.removeAnnotation(store.selectedAnnotationId);
    }
    return;
  }

  // 左右箭头 切换图片
  if (key === "arrowleft" || key === "a") {
    goToPrevImage();
    return;
  }
  if (key === "arrowright" || key === "d") {
    goToNextImage();
    return;
  }

  // 工具切换
  const tool = toolKeyMap[key];
  if (tool) {
    // 检查当前任务是否支持该工具
    const allTaskTools = [...baseTools, ...taskTools.value, ...(task.value?.task_type === "classification" ? clsTools : [])];
    if (allTaskTools.some(t => t.name === tool)) {
      activeTool.value = tool;
    }
    return;
  }
}

const shortcutHint = computed(() => {
  const tt = task.value?.task_type ?? "detection";
  const tools = [...baseTools, ...(taskToolMap[tt] ?? []), ...(tt === "classification" ? clsTools : [])];
  return tools.map(t => {
    const k = Object.entries(toolKeyMap).find(([_, v]) => v === t.name)?.[0];
    return k ? `${t.label}[${k.toUpperCase()}]` : t.label;
  }).join("  ") + "  ←[←/A]  →[→/D]  删除[Del]  保存[Ctrl+S]";
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

// ==================== AI 自动标注 ====================
const showAiAnnotateModal = ref(false);
const aiAnnotating = ref(false);
const aiAnnotateMode = ref<"current" | "all">("current");
const aiAnnotateProgress = ref({ current: 0, total: 0, message: "" });
const aiModelPath = ref("");
const ocrDetModelPath = ref("");
const ocrRecModelPath = ref("");
const ocrClsModelPath = ref("");
const ocrDictPath = ref("");

function openAiAnnotate() {
  aiModelPath.value = "";
  ocrDetModelPath.value = "";
  ocrRecModelPath.value = "";
  ocrClsModelPath.value = "";
  ocrDictPath.value = "";
  aiAnnotateMode.value = "current";
  aiAnnotateProgress.value = { current: 0, total: 0, message: "" };
  showAiAnnotateModal.value = true;
}

async function pickAiModelFile(type: string) {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const filters = type === "dict"
    ? [{ name: "Text File", extensions: ["txt"] }, { name: "All Files", extensions: ["*"] }]
    : [{ name: "ONNX Model", extensions: ["onnx"] }, { name: "All Files", extensions: ["*"] }];
  const selected = await open({ multiple: false, filters, title: `选择${type}模型` });
  if (selected) {
    const p = typeof selected === "string" ? selected : Array.isArray(selected) ? selected[0] : "";
    if (type === "model") aiModelPath.value = p;
    else if (type === "det") ocrDetModelPath.value = p;
    else if (type === "rec") ocrRecModelPath.value = p;
    else if (type === "cls") ocrClsModelPath.value = p;
    else if (type === "dict") ocrDictPath.value = p;
  }
}

async function handleAiAnnotate() {
  if (!task.value) return;
  const t = task.value;

  // 验证模型路径
  if (t.task_type === "ocr") {
    if (!ocrDetModelPath.value || !ocrRecModelPath.value || !ocrDictPath.value) {
      message.warning("OCR 任务需要指定检测模型、识别模型和字典文件");
      return;
    }
  } else {
    if (!aiModelPath.value) {
      message.warning("请选择模型 ONNX 文件");
      return;
    }
  }

  // 如果是全部标注且有已标注的图片，警告覆盖
  if (aiAnnotateMode.value === "all") {
    const annotatedCount = Object.values(store.imageAnnotationMap).filter(Boolean).length;
    if (annotatedCount > 0) {
      const confirmed = window.confirm(`当前有 ${annotatedCount} 张图片已标注，AI 标注所有图片将覆盖已有的标注，是否继续？`);
      if (!confirmed) return;
    }
  }

  aiAnnotating.value = true;
  aiAnnotateProgress.value = { current: 0, total: 0, message: "正在加载模型..." };

  // 监听进度事件
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen("auto-annotate-progress", (event) => {
    const p = event.payload as { current: number; total: number; message: string };
    aiAnnotateProgress.value = p;
  });

  try {
    const ocrModelsJson = t.task_type === "ocr" ? JSON.stringify({
      det: ocrDetModelPath.value,
      cls: ocrClsModelPath.value,
      rec: ocrRecModelPath.value,
      dict: ocrDictPath.value,
    }) : null;

    const result = await invoke<{ total_images: number; annotated_images: number; total_annotations: number }>("auto_annotate", {
      request: {
        image_folder: t.image_folder,
        task_type: t.task_type,
        classes: (t.classes ?? []).map(c => ({ id: c.id, name: c.name })),
        model_path: aiModelPath.value || null,
        ocr_models: ocrModelsJson,
      },
    });

    message.success(`自动标注完成！共 ${result.annotated_images} 张图片，${result.total_annotations} 个标注`);
    showAiAnnotateModal.value = false;
  } catch (e) {
    message.error(`自动标注失败: ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    unlisten();
    aiAnnotating.value = false;
  }
}

// ==================== 生命周期 ====================
onMounted(async () => {
  await loadTaskImages();
  if (taskImages.value.length > 0) {
    await loadImageFromPath(taskImages.value[0].path);
  }
  document.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  document.removeEventListener("keydown", onKeyDown);
});

// ==================== 图片 ====================
// ==================== 图片目录：按需懒加载标注状态 ====================
// 为每张图单独发起标注查询，查完才更新（不阻塞首图展示）
async function loadAnnotationStatus(idx: number) {
  const img = taskImages.value[idx];
  if (!img || store.imageAnnotationMap[img.path] !== undefined) return; // 已有则跳过
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const anns = await invoke<any[]>("load_annotations_for_image", { imagePath: img.path });
    store.imageAnnotationMap[img.path] = anns.length > 0;
  } catch { store.imageAnnotationMap[img.path] = false; }
}

async function loadTaskImages() {
  if (!task.value) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<{ folder_path: string; images: ImageInfo[] }>("load_folder", {
      folderPath: task.value!.image_folder,
    });
    taskImages.value = result.images;
    // 清空旧任务的标注状态缓存
    store.imageAnnotationMap = {};
    // 同步总图片数到项目统计
    if (task.value) {
      task.value.stats.total_images = result.images.length;
      projectStore.saveProject();
    }

    // 立即显示第一张图（不等待标注状态）
    if (result.images.length > 0) {
      await loadImageFromPath(result.images[0].path);
    }

    // 批量查询所有图片的标注状态（不阻塞 UI）
    (async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const statuses = await invoke<{ path: string; has_annotations: boolean }[]>("get_annotation_statuses", {
          imageFolder: task.value!.image_folder,
        });
        for (const s of statuses) {
          store.imageAnnotationMap[s.path] = s.has_annotations;
        }
        // 同步项目统计
        const annotated = Object.values(store.imageAnnotationMap).filter(Boolean).length;
        if (task.value) {
          task.value.stats.annotated_images = annotated;
          projectStore.saveProject();
        }
      } catch (e) {
        console.error("批量查询标注状态失败:", e);
        // fallback: 只查当前图
        if (result.images.length > 0) await loadAnnotationStatus(0);
      }
    })();
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
  if (currentImageIndex.value < taskImages.value.length - 1) {
    goToImage(currentImageIndex.value + 1);
  } else {
    message.warning("已经是最后一张图片");
  }
}

function goToPrevImage() {
  if (currentImageIndex.value > 0) {
    goToImage(currentImageIndex.value - 1);
  } else {
    message.warning("已经是第一张图片");
  }
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
    store.imageAnnotationMap[store.imagePath] = store.annotations.length > 0;
    const annotated = Object.values(store.imageAnnotationMap).filter(Boolean).length;
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

// ==================== 标注编辑 ====================
const annEditVisible = ref(false);
const annEditId = ref<string | null>(null);
const annEditType = ref<string>("");
const annEditClassId = ref<number | null>(null);
const annEditText = ref("");

const classOptions = computed(() =>
  (task.value?.classes ?? []).map(c => ({ label: c.name, value: c.id }))
);

function openAnnEdit(id: string) {
  const ann = store.annotations.find(a => a.id === id);
  if (!ann) return;
  annEditId.value = id;
  annEditType.value = ann.type;
  annEditClassId.value = "class_id" in ann ? ann.class_id : null;
  annEditText.value = ann.type === "Ocr" ? (ann as any).text ?? "" : "";
  annEditVisible.value = true;
}

function confirmAnnEdit() {
  const id = annEditId.value;
  if (!id) return;
  const ann = store.annotations.find(a => a.id === id);
  if (!ann) return;
  // 更新类别
  if (annEditClassId.value !== null && "class_id" in ann) {
    store.updateAnnotation(id, { class_id: annEditClassId.value } as any);
  }
  // 更新OCR文本
  if (ann.type === "Ocr" && annEditText.value !== (ann as any).text) {
    store.updateAnnotation(id, { text: annEditText.value } as any);
  }
  annEditVisible.value = false;
}

// ==================== 统计 ====================
const annotatedCount = computed(() =>
  Object.values(store.imageAnnotationMap).filter(Boolean).length
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

.shortcut-hint {
  font-size: 10px;
  color: var(--text-dim);
  margin-right: 12px;
  white-space: nowrap;
  opacity: 0.7;
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

/* 标注编辑弹窗 */
.modal-body-edit {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.modal-body-edit .field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.modal-body-edit .field-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}
.modal-body-edit .modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

/* ---- AI 标注按钮 ---- */
.ai-btn {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 10px;
  font-weight: 600;
  color: var(--accent);
  margin-top: 4px;
}
.ai-btn:hover {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
}

/* ---- AI 标注弹窗 ---- */
.modal-body-export {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.modal-body-export .field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.modal-body-export .field-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}
.mode-switch {
  display: flex;
}
.dir-row {
  display: flex;
  gap: 8px;
  align-items: center;
}
.dir-row .n-input {
  flex: 1;
}
.export-progress {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}
.progress-bar-outer {
  height: 6px;
  background: var(--bg-elevated);
  border-radius: 3px;
  overflow: hidden;
}
.progress-bar-inner {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.3s ease;
}
.progress-text {
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
