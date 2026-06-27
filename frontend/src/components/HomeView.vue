<template>
  <div class="home">
    <AppHeader>
      <template #center>
      </template>
    </AppHeader>

    <main class="home-content">
      <div v-if="projectStore.tasks.length === 0" class="empty-state">
        <div class="empty-icon">📋</div>
        <h2>还没有标注任务</h2>
        <p>点击下方按钮创建第一个标注任务</p>
        <NButton type="primary" size="large" @click="showCreateModal = true">
          <template #icon><Plus /></template>
          创建第一个任务
        </NButton>
      </div>

      <template v-else>
        <!-- 筛选栏 -->
        <div class="filter-bar" style="display:flex;flex-direction:row;align-items:center;gap:16px;">
        <NInput v-model:value="filterText" placeholder="搜索任务名称…" size="small" clearable style="width:240px;flex-shrink:0" />
        <NSelect v-model:value="filterType" :options="filterTypeOptions" placeholder="全部类型" size="small" clearable style="width:140px;flex-shrink:0" />
        <NSelect v-model:value="filterSort" :options="filterSortOptions" size="small" style="width:140px;flex-shrink:0" />
        </div>

        <template v-if="filteredTasks.length > 0">
          <div class="card-list">
<div v-for="task in pagedTasks" :key="task.id" class="task-card" @click="openTask(task.id)">
            <!-- 任务名称 + 类型标签 -->
            <div class="card-top">
              <h3 class="card-title">{{ task.name }}</h3>
              <div class="card-badges">
                <span class="card-type-badge" :style="{ color: typeColor(task.task_type), background: typeColor(task.task_type) + '18' }">
                  {{ TASK_TYPE_ICONS[task.task_type] }} {{ TASK_TYPE_LABELS[task.task_type] }}
                </span>
                <span v-if="task.task_type === 'classification'" class="card-mode-tag" :class="task.config?.classification_mode ?? 'multi'">
                  {{ task.config?.classification_mode === 'single' ? '单标签' : '多标签' }}
                </span>
              </div>
            </div>

            <!-- 中间内容（flex: 1 填充空间，按钮始终贴底） -->
            <div class="card-content">
              <div class="card-info">
              <div class="info-row">
                <ImageIcon :size="13" class="info-icon" />
                <span>{{ task.stats.total_images }} 张图片</span>
              </div>
              <div class="info-row">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="info-icon"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                <span class="info-path">{{ shortPath(task.image_folder) }}</span>
              </div>
              <div class="info-row info-time">
                <span class="time-label">创建</span>
                <span class="time-val">{{ formatTime(task.created_at) }}</span>
                <span class="time-sep">·</span>
                <span class="time-label">编辑</span>
                <span class="time-val">{{ formatTime(task.updated_at) }}</span>
              </div>
            </div>

            <!-- 进度条 -->
            <div class="card-progress">
              <div class="progress-stats">
                <span>标注进度</span>
                <span class="progress-numbers">{{ task.stats.annotated_images }} / {{ task.stats.total_images }} · {{ progressOf(task) }}%</span>
              </div>
              <div class="progress-bar-bg">
                <div class="progress-bar-fill" :style="{ width: progressOf(task) + '%' }" :class="{ 'card-bar-done': progressOf(task) >= 100 }" />
              </div>
            </div>
            </div>

            <!-- 底部操作栏 -->
            <div class="card-footer" @click.stop>
              <button class="footer-btn" title="编辑" @click="openTask(task.id)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                <span>编辑</span>
              </button>
              <button class="footer-btn" title="导出" @click="openExport(task)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
                <span>导出</span>
              </button>
              <button class="footer-btn" title="AI自动标注" @click="openAutoAnnotate(task)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2a10 10 0 1 0 0 20 10 10 0 1 0 0-20z"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>
                <span>标注</span>
              </button>
              <button class="footer-btn btn-danger" title="删除" @click="confirmDelete(task)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
                <span>删除</span>
              </button>
            </div>
          </div>
        </div>
        <!-- 翻页 -->
        <div v-if="filteredTasks.length > 0" class="pagination">
          <NPagination
            size="small"
            show-quick-jumper
            :page="currentPage"
            :page-count="totalPages"
            :page-size="pageSize"
            :page-sizes="[8, 12, 16, 24]"
            :item-count="filteredTasks.length"
            show-size-picker
            @update:page="goToPage"
            @update:page-size="(s) => { pageSize = s; currentPage = 1; }"
          >
            <template #prefix>
              <span style="font-size:12px;color:var(--text-secondary);margin-right:8px;white-space:nowrap;">共 {{ filteredTasks.length }} 项</span>
            </template>
          </NPagination>
        </div>
        </template>
        <div v-else class="empty-filter">
          <p>没有匹配的任务，请调整筛选条件</p>
        </div>
      </template>
    </main>

    <NButton v-if="projectStore.tasks.length > 0" class="fab" @click="showCreateModal = true" title="新建任务" circle type="primary" size="large">
      <template #icon><Plus :size="24" /></template>
    </NButton>

    <NDrawer v-model:show="showCreateModal" :width="400" placement="right">
      <NDrawerContent title="新建标注任务" :native-scrollbar="false">
        <div class="drawer-body">
          <div class="field">
            <label class="field-label">任务名称</label>
            <NInput v-model:value="formName" placeholder="例如：行人检测、车牌识别" size="small" />
          </div>
          <div class="field">
            <label class="field-label">任务类型</label>
            <div class="type-grid">
              <div v-for="tt in TASK_TYPES" :key="tt.value" class="type-option" :class="{ active: formType === tt.value }" @click="formType = tt.value">
                <span class="type-icon">{{ TASK_TYPE_ICONS[tt.value] }}</span>
                <span class="type-name">{{ tt.label }}</span>
                <span class="type-desc">{{ tt.desc }}</span>
              </div>
            </div>
          </div>
          <div v-if="formType === 'classification'" class="field">
            <label class="field-label">标注模式</label>
            <div class="mode-switch">
              <NButtonGroup size="small">
                <NButton :type="formClassMode === 'single' ? 'primary' : 'default'" @click="formClassMode = 'single'">单标签</NButton>
                <NButton :type="formClassMode === 'multi' ? 'primary' : 'default'" @click="formClassMode = 'multi'">多标签</NButton>
              </NButtonGroup>
            </div>
          </div>
          <div class="field">
            <label class="field-label">图片文件夹</label>
            <NInput v-model:value="formFolder" placeholder="点击选择文件夹..." readonly size="small" @click="pickFolder">
              <template #prefix><FolderOpen :size="14" /></template>
            </NInput>
          </div>
        </div>
        <template #footer>
          <div class="drawer-footer">
            <NButton size="small" @click="showCreateModal = false">取消</NButton>
            <NButton type="primary" size="small" :disabled="!canCreate" @click="handleCreate">创建</NButton>
          </div>
        </template>
      </NDrawerContent>
    </NDrawer>

    <NModal v-model:show="showDeleteModal" preset="dialog" title="删除任务" positive-text="删除" negative-text="取消" type="error" @positive-click="handleDelete">
      确定要删除任务「{{ deleteTarget?.name }}」吗？此操作不可恢复。
    </NModal>

    <!-- 导出弹窗 -->
    <NModal v-model:show="showExportModal" preset="card" title="导出标注" :mask-closable="true" style="width: 420px">
      <div class="modal-body-export">
        <div class="field">
          <label class="field-label">任务</label>
          <div class="export-task-name">{{ exportTarget?.name }}</div>
        </div>
        <div class="field">
          <label class="field-label">导出格式</label>
          <NSelect v-model:value="exportFormat" :options="exportFormatOptions" size="small" :disabled="exporting" />
        </div>
        <div class="field">
          <label class="field-label">导出目录</label>
          <div class="dir-row">
            <NInput v-model:value="exportDir" readonly size="small" placeholder="选择导出目录..." :disabled="exporting" />
            <NButton size="small" @click="pickExportDir" :disabled="exporting">选择</NButton>
          </div>
        </div>
        <div v-if="exporting" class="export-progress">
          <div class="progress-bar-outer">
            <div class="progress-bar-inner" :style="{ width: (exportProgress.total > 0 ? exportProgress.current / exportProgress.total * 100 : 0) + '%' }" />
          </div>
          <span class="progress-text">{{ exportProgress.message }} {{ exportProgress.total > 0 ? exportProgress.current + '/' + exportProgress.total : '' }}</span>
        </div>
      </div>
      <template #footer>
        <div class="drawer-footer">
          <NButton size="small" @click="showExportModal = false" :disabled="exporting">取消</NButton>
          <NButton size="small" type="primary" @click="handleExport" :loading="exporting">导出</NButton>
        </div>
      </template>
    </NModal>

    <!-- 自动标注弹窗 -->
    <NModal v-model:show="showAutoAnnotateModal" preset="card" title="AI 自动标注" :mask-closable="true" style="width: 480px">
      <div class="modal-body-export">
        <div class="field">
          <label class="field-label">任务</label>
          <div class="export-task-name">{{ autoAnnotateTarget?.name }} ({{ TASK_TYPE_LABELS[autoAnnotateTarget?.task_type ?? ''] }})</div>
        </div>
        <div v-if="autoAnnotateTarget?.task_type === 'ocr'" class="field">
          <label class="field-label">检测模型 ONNX</label>
          <div class="dir-row">
            <NInput v-model:value="ocrDetModelPath" size="small" placeholder="选择 det 模型..." />
            <NButton size="small" @click="pickModelFile('det')">选择</NButton>
          </div>
          <label class="field-label" style="margin-top:4px">识别模型 ONNX</label>
          <div class="dir-row">
            <NInput v-model:value="ocrRecModelPath" size="small" placeholder="选择 rec 模型..." />
            <NButton size="small" @click="pickModelFile('rec')">选择</NButton>
          </div>
          <label class="field-label" style="margin-top:4px">分类模型 ONNX (可选)</label>
          <div class="dir-row">
            <NInput v-model:value="ocrClsModelPath" size="small" placeholder="选择 cls 模型..." />
            <NButton size="small" @click="pickModelFile('cls')">选择</NButton>
          </div>
          <label class="field-label" style="margin-top:4px">字典文件</label>
          <div class="dir-row">
            <NInput v-model:value="ocrDictPath" size="small" placeholder="选择 dict.txt..." />
            <NButton size="small" @click="pickModelFile('dict')">选择</NButton>
          </div>
        </div>
        <div v-else class="field">
          <label class="field-label">模型 ONNX 路径</label>
          <div class="dir-row">
            <NInput v-model:value="modelPath" size="small" placeholder="选择模型文件..." />
            <NButton size="small" @click="pickModelFile('model')">选择</NButton>
          </div>
        </div>
        <div v-if="autoAnnotating" class="export-progress">
          <div class="progress-bar-outer">
            <div class="progress-bar-inner" :style="{ width: (autoAnnotateProgress.total > 0 ? autoAnnotateProgress.current / autoAnnotateProgress.total * 100 : 0) + '%' }" />
          </div>
          <span class="progress-text">{{ autoAnnotateProgress.message }} {{ autoAnnotateProgress.total > 0 ? autoAnnotateProgress.current + '/' + autoAnnotateProgress.total : '' }}</span>
        </div>
      </div>
      <template #footer>
        <div class="drawer-footer">
          <NButton size="small" @click="showAutoAnnotateModal = false" :disabled="autoAnnotating">取消</NButton>
          <NButton size="small" type="primary" @click="handleAutoAnnotate" :loading="autoAnnotating">开始标注</NButton>
        </div>
      </template>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { NButton, NInput, NSelect, NPagination, NDrawer, NDrawerContent, NModal, NButtonGroup, useMessage } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { Plus, FolderOpen } from "lucide-vue-next";
import { Image as ImageIcon } from "lucide-vue-next";
import { useProjectStore } from "@/stores/project";
import { TASK_TYPE_LABELS, TASK_TYPE_ICONS, type TaskType, type Task, type ClassificationMode } from "@/utils/taskTypes";
import AppHeader from "./AppHeader.vue";

const projectStore = useProjectStore();
const message = useMessage();

const TASK_TYPES: { value: TaskType; label: string; desc: string }[] = [
  { value: "classification", label: "分类", desc: "为图片分配类别标签（支持单标签/多标签）" },
  { value: "detection", label: "目标检测", desc: "用矩形框标注目标位置" },
  { value: "rotated_detection", label: "旋转框检测", desc: "用旋转矩形框标注倾斜目标" },
  { value: "keypoint", label: "关键点检测", desc: "标注关键点和包围框" },
  { value: "segmentation", label: "实例分割", desc: "用多边形精确勾勒目标轮廓" },
  { value: "ocr", label: "OCR 文本识别", desc: "标注图片中的文字区域" },
];

const TYPE_COLORS: Record<TaskType, string> = {
  classification: "#6366f1",
  detection: "#3b82f6",
  rotated_detection: "#8b5cf6",
  keypoint: "#eab308",
  segmentation: "#22c55e",
  ocr: "#06b6d4",
};
function typeColor(tt: TaskType): string {
  return TYPE_COLORS[tt] ?? "#6b7280";
}

const showCreateModal = ref(false);
const formName = ref("");
const formType = ref<TaskType>("detection");
const formClassMode = ref<ClassificationMode>("single");
const formFolder = ref("");

const canCreate = computed(() => formName.value.trim() && formFolder.value);

async function pickFolder() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({ directory: true, title: "选择图片文件夹" });
  if (selected) formFolder.value = selected;
}

async function handleCreate() {
  if (!canCreate.value) return;
  const name = formName.value.trim();
  if (projectStore.tasks.some(t => t.name === name)) {
    message.warning(`任务名「${name}」已存在`);
    return;
  }
  await projectStore.createTask(name, formType.value, formFolder.value, formType.value === 'classification' ? formClassMode.value : undefined);
  formName.value = "";
  formFolder.value = "";
  formClassMode.value = "single";
  showCreateModal.value = false;
}

const showDeleteModal = ref(false);
const deleteTarget = ref<Task | null>(null);

function confirmDelete(task: Task) {
  deleteTarget.value = task;
  showDeleteModal.value = true;
}

async function handleDelete() {
  if (deleteTarget.value) {
    await projectStore.deleteTask(deleteTarget.value.id);
  }
  deleteTarget.value = null;
}

// ==================== 导出 ====================
const EXPORT_FORMATS: Record<string, { value: string; label: string }[]> = {
  detection: [{ value: "yolo", label: "YOLO (.txt)" }],
  rotated_detection: [{ value: "yolo_obb", label: "YOLO OBB (.txt)" }],
  segmentation: [{ value: "yolo", label: "YOLO (.txt)" }, { value: "coco_json", label: "COCO JSON" }],
  keypoint: [{ value: "coco_json", label: "COCO JSON" }],
  ocr: [{ value: "paddleocr", label: "PaddleOCR" }],
  classification: [{ value: "csv", label: "CSV" }, { value: "yolo", label: "YOLO (.txt)" }],
};

const showExportModal = ref(false);
const exportTarget = ref<Task | null>(null);
const exportFormat = ref("");
const exportDir = ref("");
const exportFormatOptions = computed(() => EXPORT_FORMATS[exportTarget.value?.task_type ?? ""] ?? []);
const exporting = ref(false);
const exportProgress = ref({ current: 0, total: 0, message: "" });

function openExport(task: Task) {
  exportTarget.value = task;
  const formats = EXPORT_FORMATS[task.task_type];
  exportFormat.value = formats?.[0]?.value ?? "";
  exportDir.value = `${task.image_folder}/export_${exportFormat.value}`;
  exportProgress.value = { current: 0, total: 0, message: "" };
  showExportModal.value = true;
}

async function pickExportDir() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({ directory: true, title: "选择导出目录" });
  if (selected) exportDir.value = selected;
}

async function handleExport() {
  if (!exportTarget.value || !exportFormat.value || !exportDir.value) return;
  const t = exportTarget.value;
  if (t.stats.annotated_images < t.stats.total_images) {
    message.warning(`标注未完成 (${t.stats.annotated_images}/${t.stats.total_images})，无法导出`);
    return;
  }
  exporting.value = true;
  exportProgress.value = { current: 0, total: 0, message: "准备导出..." };
  // 监听进度事件
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen("export-progress", (event) => {
    const p = event.payload as { current: number; total: number; message: string };
    exportProgress.value = p;
  });
  try {
    const outputPath = await invoke<string>("export_annotations", {
      request: {
        image_folder: t.image_folder,
        output_dir: exportDir.value,
        task_type: t.task_type,
        export_format: exportFormat.value,
        classes: (t.classes ?? []).map(c => ({ id: c.id, name: c.name })),
      },
    });
    message.success(`导出成功: ${outputPath}`);
    showExportModal.value = false;
  } catch (e) {
    message.error(`导出失败: ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    unlisten();
    exporting.value = false;
  }
}

function openTask(id: string) {
  projectStore.openTask(id);
}

// ==================== AI 自动标注 ====================
const showAutoAnnotateModal = ref(false);
const autoAnnotateTarget = ref<Task | null>(null);
const autoAnnotating = ref(false);
const autoAnnotateProgress = ref({ current: 0, total: 0, message: "" });
const modelPath = ref("");
const ocrDetModelPath = ref("");
const ocrRecModelPath = ref("");
const ocrClsModelPath = ref("");
const ocrDictPath = ref("");

function openAutoAnnotate(task: Task) {
  autoAnnotateTarget.value = task;
  modelPath.value = "";
  ocrDetModelPath.value = "";
  ocrRecModelPath.value = "";
  ocrClsModelPath.value = "";
  ocrDictPath.value = "";
  autoAnnotateProgress.value = { current: 0, total: 0, message: "" };
  showAutoAnnotateModal.value = true;
}

async function pickModelFile(type: string) {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    multiple: false,
    filters: [{ name: "ONNX Model", extensions: ["onnx"] }, { name: "Text File", extensions: ["txt"] }, { name: "All Files", extensions: ["*"] }],
    title: `选择${type}模型文件`,
  });
  if (selected) {
    const path = typeof selected === "string" ? selected : Array.isArray(selected) ? selected[0] : "";
    if (type === "model") modelPath.value = path;
    else if (type === "det") ocrDetModelPath.value = path;
    else if (type === "rec") ocrRecModelPath.value = path;
    else if (type === "cls") ocrClsModelPath.value = path;
    else if (type === "dict") ocrDictPath.value = path;
  }
}

async function handleAutoAnnotate() {
  if (!autoAnnotateTarget.value) return;
  const t = autoAnnotateTarget.value;

  // 验证模型路径
  if (t.task_type === "ocr") {
    if (!ocrDetModelPath.value || !ocrRecModelPath.value || !ocrDictPath.value) {
      message.warning("OCR 任务需要指定检测模型、识别模型和字典文件");
      return;
    }
  } else {
    if (!modelPath.value) {
      message.warning("请选择模型 ONNX 文件");
      return;
    }
  }

  autoAnnotating.value = true;
  autoAnnotateProgress.value = { current: 0, total: 0, message: "正在加载模型..." };

  // 监听进度事件
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen("auto-annotate-progress", (event) => {
    const p = event.payload as { current: number; total: number; message: string };
    autoAnnotateProgress.value = p;
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
        model_path: modelPath.value || null,
        ocr_models: ocrModelsJson,
      },
    });

    message.success(`自动标注完成！共 ${result.annotated_images} 张图片，${result.total_annotations} 个标注`);
    // 刷新任务列表统计
    await projectStore.loadProject();
    showAutoAnnotateModal.value = false;
  } catch (e) {
    message.error(`自动标注失败: ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    unlisten();
    autoAnnotating.value = false;
  }
}

// ==================== 筛选 ====================
const filterTypeOptions = [
  { label: "全部类型", value: "" as string },
  ...TASK_TYPES.map(tt => ({ label: tt.label, value: tt.value })),
];
const filterSortOptions = [
  { label: "最新创建", value: "newest" },
  { label: "最早创建", value: "oldest" },
  { label: "名称排序", value: "name" },
];
const filterText = computed({
  get: () => projectStore.filterText,
  set: (v: string) => projectStore.filterText = v,
});
const filterType = computed({
  get: () => projectStore.filterType,
  set: (v: string) => projectStore.filterType = v,
});
const filterSort = computed({
  get: () => projectStore.filterSort,
  set: (v: string) => projectStore.filterSort = v,
});

const filteredTasks = computed(() => {
  let list = projectStore.tasks;
  if (filterText.value) {
    const q = filterText.value.toLowerCase();
    list = list.filter(t => t.name.toLowerCase().includes(q));
  }
  if (filterType.value) {
    list = list.filter(t => t.task_type === filterType.value);
  }
  if (filterSort.value === "newest") {
    list = [...list].sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
  } else if (filterSort.value === "oldest") {
    list = [...list].sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime());
  } else if (filterSort.value === "name") {
    list = [...list].sort((a, b) => a.name.localeCompare(b.name));
  }
  return list;
});

// ==================== 分页 ====================
const pageSize = ref(12);
const currentPage = ref(1);
const totalPages = computed(() => Math.max(1, Math.ceil(filteredTasks.value.length / pageSize.value)));
const pagedTasks = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  return filteredTasks.value.slice(start, start + pageSize.value);
});

function goToPage(n: number) {
  currentPage.value = Math.max(1, Math.min(totalPages.value, n));
}

watch([pageSize, filterText, filterType, filterSort], () => { currentPage.value = 1; });

function progressOf(task: Task): number {
  if (task.stats.total_images === 0) return 0;
  return Math.min(100, Math.round((task.stats.annotated_images / task.stats.total_images) * 100));
}

function shortPath(p: string): string {
  if (!p) return "";
  const parts = p.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || parts[parts.length - 2] || p;
}

function formatTime(iso: string): string {
  if (!iso) return "-";
  try {
    const d = new Date(iso);
    return d.toLocaleDateString("zh-CN", { year: "numeric", month: "2-digit", day: "2-digit" });
  } catch {
    return iso;
  }
}
</script>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.home-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 16px 48px 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  gap: 12px;
  padding: 32px 48px;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 8px;
}

.empty-state h2 {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}

.empty-state p {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}





/* ---- 筛选栏 ---- */
.filter-bar {
  display: flex;
  flex-direction: row;
  gap: 16px;
  align-items: center;
  padding: 0;
  margin-bottom: 0;
  flex-shrink: 0;
}
.filter-input {
  flex: 1;
  min-width: 0;
}
.filter-select {
  width: 140px;
  flex-shrink: 0;
}

.card-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 14px;
  align-content: start;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  margin: 12px 0;
  padding: 0;
}

.task-card {
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  padding: 14px 16px 0;
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.task-card:hover {
  border-color: var(--accent);
  box-shadow: 0 2px 12px color-mix(in srgb, var(--accent) 15%, transparent);
}
.card-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}
.card-badges {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.card-type-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 6px;
  line-height: 1.3;
  white-space: nowrap;
  flex-shrink: 0;
}
.card-mode-tag {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
  line-height: 1.3;
  white-space: nowrap;
}
.card-mode-tag.single {
  background: #22c55e18;
  color: #22c55e;
}
.card-mode-tag.multi {
  background: #3b82f618;
  color: #3b82f6;
}
.card-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.info-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}
.info-time {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-dim);
}
.info-time .time-sep {
  opacity: 0.5;
}
.info-time .time-val {
  font-variant-numeric: tabular-nums;
}
.info-icon {
  flex-shrink: 0;
  color: var(--text-dim);
  opacity: 1;
}
.info-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.card-progress {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.progress-stats {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: var(--text-secondary);
}
.progress-numbers {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}
.progress-bar-bg {
  height: 5px;
  background: var(--bg-elevated);
  border-radius: 3px;
  overflow: hidden;
}
.progress-bar-fill {
  height: 100%;
  border-radius: 3px;
  background: var(--accent);
  transition: width 0.4s ease;
}
.progress-bar-fill.card-bar-done {
  background: var(--success);
}
.card-footer {
  display: flex;
  gap: 0;
  border-top: 1px solid var(--border-subtle);
  margin-top: 0;
}
.footer-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 7px 0;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.footer-btn + .footer-btn {
  border-left: 1px solid var(--border-subtle);
}
.footer-btn:hover {
  background: var(--bg-hover);
  color: var(--accent);
}
.footer-btn.btn-danger:hover {
  color: var(--danger);
}

.fab {
  position: fixed;
  bottom: 28px;
  right: 28px;
  width: 52px;
  height: 52px;
  border-radius: 50%;
  background: var(--accent);
  color: #fff;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 4px 16px rgba(0,0,0,0.3);
  transition: transform 0.15s, background 0.15s;
  z-index: 100;
}

.fab:hover {
  transform: scale(1.1);
  filter: brightness(1.15);
}

/* ---- 空筛选结果 ---- */
.empty-filter {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--text-dim);
  font-size: 14px;
}
.empty-filter p {
  padding: 32px;
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
}

/* ---- 翻页 ---- */
.pagination {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  padding: 4px 12px;
  background: var(--bg-panel);
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}


/* ---- 导出弹窗 ---- */
.modal-body-export {
  display: flex;
  flex-direction: column;
  gap: 12px;
  font-size: 13px;
}
.modal-body-export :deep(.n-alert) {
  font-size: 12px !important;
}
.modal-body-export :deep(.n-alert__title) {
  font-size: 12px !important;
  font-weight: 600;
}
.modal-body-export :deep(.n-alert__content) {
  font-size: 12px !important;
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
.export-task-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
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
.dir-row {
  display: flex;
  gap: 8px;
  align-items: center;
}
.dir-row .n-input {
  flex: 1;
}

.drawer-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}

.type-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.type-option {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 12px;
  border: 1px solid var(--border-subtle);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
}

.type-option:hover {
  border-color: var(--text-dim);
  background: var(--bg-hover);
}

.type-option.active {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

.mode-switch {
  display: flex;
  gap: 0;
  border: 1px solid var(--border-subtle);
  border-radius: 6px;
  overflow: hidden;
  align-self: flex-start;
}
.mode-btn {
  padding: 6px 20px;
  font-size: 12px;
  font-weight: 600;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.mode-btn.active {
  background: var(--accent);
  color: #fff;
}
.mode-btn:not(.active):hover {
  background: var(--bg-hover);
}

.type-icon {
  font-size: 18px;
}

.type-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.type-desc {
  font-size: 11px;
  color: var(--text-dim);
}

.drawer-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
