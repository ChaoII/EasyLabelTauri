<template>
  <div class="model-page">
    <div class="page-header">
      <h2>模型管理</h2>
      <NButton size="small" type="primary" @click="showAddModal = true">添加模型</NButton>
    </div>

    <div v-if="modelStore.models.length === 0" class="empty-state">
      <p>暂无模型配置，点击「添加模型」开始</p>
    </div>

    <div v-else class="model-list">
      <div v-for="m in modelStore.models" :key="m.id" class="model-card">
        <div class="model-top">
          <span class="model-type-badge" :style="{ background: typeColor(m.task_type) + '22', color: typeColor(m.task_type) }">{{ TASK_TYPE_LABELS[m.task_type as TaskType] || m.task_type }}</span>
          <span class="model-runtime" :class="m.runtime">{{ m.runtime === 'gpu' ? 'GPU' : 'CPU' }}</span>
          <NButton quaternary circle size="tiny" @click="modelStore.removeModel(m.id)" title="删除">
            <template #icon><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></template>
          </NButton>
        </div>
        <div class="model-name">{{ m.name }}</div>
        <div class="model-path">{{ m.task_type === 'ocr' ? m.ocr_det : m.model_path }}</div>
        <div class="model-time">{{ m.created_at }}</div>
      </div>
    </div>

    <!-- 添加模型弹窗 -->
    <NModal v-model:show="showAddModal" preset="card" title="添加模型" :mask-closable="true" style="width: 440px">
      <div class="modal-body-export">
        <div class="field">
          <label class="field-label">模型名称</label>
          <NInput v-model:value="formName" size="small" placeholder="例如：YOLOv11n 行人检测" />
        </div>
        <div class="field">
          <label class="field-label">任务类型</label>
          <NSelect v-model:value="formType" :options="TASK_TYPES_OPTIONS" size="small" />
        </div>
        <div class="field">
          <label class="field-label">运行设备</label>
          <div class="mode-switch">
            <NButtonGroup size="small">
              <NButton :type="formRuntime === 'gpu' ? 'primary' : 'default'" @click="formRuntime = 'gpu'">GPU</NButton>
              <NButton :type="formRuntime === 'cpu' ? 'primary' : 'default'" @click="formRuntime = 'cpu'">CPU</NButton>
            </NButtonGroup>
          </div>
        </div>
        <div v-if="formType === 'ocr'" class="field">
          <label class="field-label">检测模型 ONNX</label>
          <div class="dir-row"><NInput v-model:value="formDet" size="small" /><NButton size="small" @click="pickFile('det')">选择</NButton></div>
          <label class="field-label" style="margin-top:4px">识别模型 ONNX</label>
          <div class="dir-row"><NInput v-model:value="formRec" size="small" /><NButton size="small" @click="pickFile('rec')">选择</NButton></div>
          <label class="field-label" style="margin-top:4px">分类模型 (可选)</label>
          <div class="dir-row"><NInput v-model:value="formCls" size="small" /><NButton size="small" @click="pickFile('cls')">选择</NButton></div>
          <label class="field-label" style="margin-top:4px">字典文件</label>
          <div class="dir-row"><NInput v-model:value="formDict" size="small" /><NButton size="small" @click="pickFile('dict')">选择</NButton></div>
        </div>
        <div v-else class="field">
          <label class="field-label">模型 ONNX 文件</label>
          <div class="dir-row"><NInput v-model:value="formPath" size="small" /><NButton size="small" @click="pickFile('model')">选择</NButton></div>
        </div>
      </div>
      <template #footer>
        <div class="drawer-footer">
          <NButton size="small" @click="showAddModal = false">取消</NButton>
          <NButton size="small" type="primary" @click="handleAdd">确认</NButton>
        </div>
      </template>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { NButton, NInput, NSelect, NModal, NButtonGroup } from "naive-ui";
import { useModelStore, type ModelConfig } from "@/stores/models";
import { TASK_TYPE_LABELS, TASK_TYPE_ICONS, type TaskType } from "@/utils/taskTypes";

const modelStore = useModelStore();

const TASK_TYPES_OPTIONS = [
  { label: "目标检测", value: "detection" },
  { label: "旋转框检测", value: "rotated_detection" },
  { label: "实例分割", value: "segmentation" },
  { label: "关键点检测", value: "keypoint" },
  { label: "图像分类", value: "classification" },
  { label: "OCR", value: "ocr" },
];

const showAddModal = ref(false);
const formName = ref("");
const formType = ref("detection");
const formRuntime = ref<"gpu" | "cpu">("gpu");
const formPath = ref("");
const formDet = ref("");
const formRec = ref("");
const formCls = ref("");
const formDict = ref("");

const typeColors: Record<string, string> = {
  classification: "#6366f1", detection: "#3b82f6", rotated_detection: "#8b5cf6",
  keypoint: "#eab308", segmentation: "#22c55e", ocr: "#06b6d4",
};
function typeColor(tt: string): string { return typeColors[tt] ?? "#6b7280"; }

async function pickFile(type: string) {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const filters = type === "dict"
    ? [{ name: "Text", extensions: ["txt"] }]
    : [{ name: "ONNX Model", extensions: ["onnx"] }];
  const sel = await open({ multiple: false, filters, title: "选择文件" });
  if (sel) {
    const p = typeof sel === "string" ? sel : Array.isArray(sel) ? sel[0] : "";
    if (type === "model") formPath.value = p;
    else if (type === "det") formDet.value = p;
    else if (type === "rec") formRec.value = p;
    else if (type === "cls") formCls.value = p;
    else if (type === "dict") formDict.value = p;
  }
}

function handleAdd() {
  if (!formName.value.trim()) return;
  const cfg: ModelConfig = {
    id: crypto.randomUUID(),
    name: formName.value.trim(),
    task_type: formType.value,
    model_path: formPath.value,
    ocr_det: formDet.value || undefined,
    ocr_cls: formCls.value || undefined,
    ocr_rec: formRec.value || undefined,
    ocr_dict: formDict.value || undefined,
    runtime: formRuntime.value,
    fp16: true,
    created_at: new Date().toISOString(),
  };
  modelStore.addModel(cfg);
  showAddModal.value = false;
  formName.value = "";
  formPath.value = "";
  formDet.value = "";
  formRec.value = "";
  formCls.value = "";
  formDict.value = "";
}
</script>

<style scoped>
.model-page { flex: 1; overflow-y: auto; padding: 24px 32px; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.page-header h2 { font-size: 18px; font-weight: 700; color: var(--text-primary); margin: 0; }
.empty-state { display: flex; justify-content: center; padding: 60px; color: var(--text-dim); font-size: 14px; }
.model-list { display: flex; flex-direction: column; gap: 12px; }
.model-card {
  background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: 8px;
  padding: 14px 16px; display: flex; flex-direction: column; gap: 6px;
}
.model-top { display: flex; align-items: center; gap: 8px; }
.model-type-badge { font-size: 11px; font-weight: 600; padding: 2px 8px; border-radius: 4px; }
.model-runtime { font-size: 10px; padding: 1px 6px; border-radius: 3px; background: var(--bg-elevated); }
.model-runtime.gpu { color: #22c55e; background: #22c55e18; }
.model-runtime.cpu { color: #3b82f6; background: #3b82f618; }
.model-name { font-size: 14px; font-weight: 600; color: var(--text-primary); }
.model-path { font-size: 11px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.model-time { font-size: 11px; color: var(--text-dim); }
.mode-switch { display: flex; }
.dir-row { display: flex; gap: 6px; align-items: center; }
.dir-row .n-input { flex: 1; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field-label { font-size: 12px; color: var(--text-secondary); font-weight: 500; }
.modal-body-export { display: flex; flex-direction: column; gap: 12px; }
.drawer-footer { display: flex; justify-content: flex-end; gap: 8px; }
</style>