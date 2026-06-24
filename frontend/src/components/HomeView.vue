<template>
  <div class="home">
    <AppHeader>
      <template #center>
        <div class="stat-chip" v-if="projectStore.tasks.length > 0">
          <span class="stat-icon">📁</span>
          <span>{{ projectStore.tasks.length }} 个任务</span>
        </div>
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

      <div v-else class="task-grid">
        <div class="overview-bar">
          <div class="overview-stat">
            <span class="stat-num">{{ totalImages }}</span>
            <span class="stat-label">总图片</span>
          </div>
          <div class="overview-stat">
            <span class="stat-num">{{ totalAnnotated }}</span>
            <span class="stat-label">已标注</span>
          </div>
          <div class="overview-stat">
            <span class="stat-num">{{ overallProgress }}%</span>
            <span class="stat-label">总进度</span>
          </div>
        </div>

        <div class="card-list">
          <div v-for="task in pagedTasks" :key="task.id" class="task-card" @click="openTask(task.id)">
            <div class="card-header">
              <div class="card-type-badge" :style="{ background: typeColor(task.task_type) + '22', color: typeColor(task.task_type) }">
                <span class="badge-icon" aria-hidden="true">{{ TASK_TYPE_ICONS[task.task_type] }}</span>
                <span class="badge-label">{{ TASK_TYPE_LABELS[task.task_type] }}</span>
              </div>
              <div class="card-actions" @click.stop>
                <button class="icon-btn" title="删除任务" @click="confirmDelete(task)">
                  <Trash2 :size="14" />
                </button>
              </div>
            </div>

            <div class="card-body">
              <h3 class="card-title">{{ task.name }}</h3>
              <div class="card-meta">
                <div class="meta-item">
                  <FolderOpen :size="12" />
                  <span>{{ shortPath(task.image_folder) }}</span>
                </div>
                <div class="meta-item">
                  <ImageIcon :size="12" />
                  <span>{{ task.stats.total_images }} 张图片</span>
          </div>
          <!-- 翻页 -->
          <div v-if="totalPages > 1" class="pagination">
            <button class="page-btn" :disabled="currentPage <= 1" @click="goToPage(currentPage - 1)">‹</button>
            <button v-for="p in totalPages" :key="p" class="page-btn" :class="{ active: p === currentPage }" @click="goToPage(p)">{{ p }}</button>
            <button class="page-btn" :disabled="currentPage >= totalPages" @click="goToPage(currentPage + 1)">›</button>
          </div>
        </div>
            </div>

            <div class="card-progress">
              <div class="progress-header">
                <span class="progress-label">标注进度</span>
                <span class="progress-value">{{ task.stats.annotated_images }} / {{ task.stats.total_images }}</span>
              </div>
              <div class="progress-bar-bg">
                <div class="progress-bar-fill" :style="{ width: progressOf(task) + '%' }" :class="{ 'card-bar-done': progressOf(task) >= 100 }" />
              </div>
              <div class="progress-percent">{{ progressOf(task) }}%</div>
            </div>

            <div class="card-footer">
              <div class="time-info">
                <span class="time-label">创建</span>
                <span class="time-val">{{ formatTime(task.created_at) }}</span>
              </div>
              <div class="time-info">
                <span class="time-label">编辑</span>
                <span class="time-val">{{ formatTime(task.updated_at) }}</span>
              </div>
              <button class="open-btn" @click.stop="openTask(task.id)">
                开始标注
                <ArrowRight :size="12" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </main>

    <button v-if="projectStore.tasks.length > 0" class="fab" @click="showCreateModal = true" title="新建任务">
      <Plus :size="24" />
    </button>

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
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { NButton, NInput, NDrawer, NDrawerContent, NModal } from "naive-ui";
import { Plus, Trash2, FolderOpen, ArrowRight } from "lucide-vue-next";
import { Image as ImageIcon } from "lucide-vue-next";
import { useProjectStore } from "@/stores/project";
import { TASK_TYPE_LABELS, TASK_TYPE_ICONS, type TaskType, type Task } from "@/utils/taskTypes";
import AppHeader from "./AppHeader.vue";

const projectStore = useProjectStore();

const TASK_TYPES: { value: TaskType; label: string; desc: string }[] = [
  { value: "classification", label: "多标签分类", desc: "为图片分配一个或多个标签" },
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
const formFolder = ref("");

const canCreate = computed(() => formName.value.trim() && formFolder.value);

async function pickFolder() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({ directory: true, title: "选择图片文件夹" });
  if (selected) formFolder.value = selected;
}

async function handleCreate() {
  if (!canCreate.value) return;
  await projectStore.createTask(formName.value.trim(), formType.value, formFolder.value);
  formName.value = "";
  formFolder.value = "";
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

function openTask(id: string) {
  projectStore.openTask(id);
}

const totalImages = computed(() =>
  projectStore.tasks.reduce((s, t) => s + t.stats.total_images, 0)
);
const totalAnnotated = computed(() =>
  projectStore.tasks.reduce((s, t) => s + t.stats.annotated_images, 0)
);
const overallProgress = computed(() =>
  totalImages.value === 0 ? 0 : Math.round((totalAnnotated.value / totalImages.value) * 100)
);

// ==================== 分页 ====================
const PAGE_SIZE = 12;
const currentPage = ref(1);
const totalPages = computed(() => Math.max(1, Math.ceil(projectStore.tasks.length / PAGE_SIZE)));
const pagedTasks = computed(() => {
  const start = (currentPage.value - 1) * PAGE_SIZE;
  return projectStore.tasks.slice(start, start + PAGE_SIZE);
});

function goToPage(n: number) {
  currentPage.value = Math.max(1, Math.min(totalPages.value, n));
}

function progressOf(task: Task): number {
  if (task.stats.total_images === 0) return 0;
  return Math.round((task.stats.annotated_images / task.stats.total_images) * 100);
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
  overflow-y: auto;
  padding: 32px 48px 80px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  gap: 12px;
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

.overview-bar {
  display: flex;
  gap: 0;
  margin-bottom: 24px;
  padding: 0;
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  overflow: hidden;
}

.overview-stat {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 20px 16px;
  position: relative;
}

.overview-stat + .overview-stat {
  border-left: 1px solid var(--border-subtle);
}

.stat-num {
  font-size: 28px;
  font-weight: 800;
  color: var(--accent);
  line-height: 1.1;
  letter-spacing: -0.02em;
}

.stat-label {
  font-size: 12px;
  color: var(--text-dim);
  font-weight: 500;
}

.stat-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  background: var(--bg-elevated);
  border-radius: 12px;
  font-size: 12px;
  color: var(--text-secondary);
}

.stat-icon {
  font-size: 14px;
}

.card-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.task-card {
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.task-card:hover {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-type-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  line-height: 1;
}

.badge-icon {
  font-size: 1.2em;
  line-height: 1;
  flex-shrink: 0;
}

.badge-label {
  line-height: 1;
}

.card-actions {
  opacity: 0;
  transition: opacity 0.15s;
}

.task-card:hover .card-actions {
  opacity: 1;
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.card-meta {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--text-secondary);
}

.card-progress {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-label {
  font-size: 11px;
  color: var(--text-secondary);
}

.progress-value {
  font-size: 11px;
  color: var(--text-dim);
  font-variant-numeric: tabular-nums;
}

.progress-bar-bg {
  height: 8px;
  background: var(--bg-elevated);
  border-radius: 4px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  border-radius: 4px;
  background: var(--accent);
  transition: width 0.4s ease;
}

.progress-bar-fill.card-bar-done {
  background: #22c55e;
}

.progress-percent {
  font-size: 12px;
  color: var(--text-dim);
  font-weight: 600;
  text-align: right;
}

.card-footer {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-top: 8px;
  border-top: 1px solid var(--border-subtle);
}

.time-info {
  display: flex;
  align-items: center;
  gap: 4px;
}

.time-label {
  font-size: 11px;
  color: var(--text-dim);
}

.time-val {
  font-size: 11px;
  color: var(--text-secondary);
}

.open-btn {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.12s, transform 0.1s;
}

.open-btn:hover {
  filter: brightness(1.15);
  transform: scale(1.02);
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

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 4px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* ---- 翻页 ---- */
.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 4px;
  margin-top: 24px;
}
.page-btn {
  min-width: 32px;
  height: 32px;
  border-radius: 4px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.1s, border-color 0.1s, color 0.1s;
  display: flex;
  align-items: center;
  justify-content: center;
}
.page-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}
.page-btn.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.page-btn:disabled {
  opacity: 0.3;
  cursor: default;
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
