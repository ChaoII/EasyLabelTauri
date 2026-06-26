<template>
  <NModal
    v-model:show="visible"
    preset="card"
    :title="editId === null ? '添加类别' : '编辑类别'"
    :mask-closable="true"
    style="width: 400px"
    @after-leave="reset"
  >
    <div class="modal-body">
      <!-- 类别名 -->
      <div class="field">
        <label class="field-label">类别名称</label>
        <NInput
          ref="nameInputRef"
          v-model:value="formName"
          placeholder="例如：person、car、logo"
          size="small"
          :maxlength="40"
          @keydown.enter.prevent="handleConfirm"
        />
      </div>

      <!-- 颜色 -->
      <div class="field">
        <label class="field-label">颜色</label>
        <div class="color-row">
          <div class="color-presets">
            <div
              v-for="c in PRESET_COLORS"
              :key="c"
              class="color-swatch"
              :class="{ active: formColor === c }"
              :style="{ background: c }"
              @click="formColor = c"
            />
          </div>
          <label class="color-picker-btn" :style="{ background: formColor }">
            <input type="color" :value="formColor" @input="formColor = ($event.target as HTMLInputElement).value" />
            <span class="picker-icon">🎨</span>
          </label>
        </div>
        <div class="color-preview" :style="{ background: formColor }">
          {{ formName || "预览" }}
        </div>
      </div>

      <!-- 关键点（仅关键点任务显示） -->
      <div v-if="taskType === 'keypoint'" class="field">
        <label class="field-label">关键点</label>
        <div class="kp-list">
          <div v-for="(kp, i) in formKps" :key="i" class="kp-row">
            <span class="kp-idx">{{ i + 1 }}</span>
            <label class="kp-color-btn" :style="{ background: kp.color }">
              <input type="color" :value="kp.color" @input="formKps[i].color = ($event.target as HTMLInputElement).value" />
            </label>
            <NInput v-model:value="formKps[i].name" placeholder="关键点名称" size="small" :maxlength="30" />
            <NButton quaternary circle size="tiny" @click="removeKp(i)" title="删除此关键点">
              <template #icon><svg width="10" height="10" viewBox="0 0 10 10" fill="none"><path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg></template>
            </NButton>
          </div>
        </div>
        <NButton quaternary size="tiny" @click="addKp" class="kp-add-btn">
          <template #icon><svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M6 1v10M1 6h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg></template>
          添加关键点
        </NButton>
      </div>
    </div>

    <template #footer>
      <div class="modal-footer">
        <NButton
          v-if="editId !== null"
          size="small"
          class="del-btn"
          @click="handleDelete"
        >
          删除类别
        </NButton>
        <div class="spacer" />
        <NButton size="small" @click="visible = false">取消</NButton>
        <NButton size="small" type="primary" @click="handleConfirm">确认</NButton>
      </div>
    </template>
  </NModal>
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";
import { NModal, NInput, NButton } from "naive-ui";
import type { ClassDefinition } from "@/utils/types";
import type { TaskType } from "@/utils/taskTypes";

const props = defineProps<{
  taskType?: TaskType;
}>();

const KP_DEFAULT_COLORS = [
  "#FF6B6B", "#FF9F43", "#FECA57", "#9CCC65",
  "#26DE81", "#20BF6B", "#0BE881", "#0FB9B1",
  "#12CBC4", "#0ABDE3", "#2E86DE", "#3863D4",
  "#8854D0", "#A55EEA", "#D980FA", "#F78FB3",
];

const PRESET_COLORS = [
  "#FF6B6B", "#FF9F43", "#FFE66D", "#78E08F",
  "#4ECDC4", "#82CCDD", "#AA96DA", "#D980FA",
  "#F38181", "#95E1D3", "#A8D8EA", "#FCBAD3",
];

interface KpEntry {
  name: string;
  color: string;
}

const visible = ref(false);
const editId = ref<number | null>(null);
const formName = ref("");
const formColor = ref("#FF6B6B");
const formKps = ref<KpEntry[]>([]);

const nameInputRef = ref<InstanceType<typeof NInput> | null>(null);

function addKp() {
  const color = KP_DEFAULT_COLORS[formKps.value.length % KP_DEFAULT_COLORS.length];
  formKps.value.push({ name: "", color });
  nextTick(() => {
    const inputs = document.querySelectorAll(".kp-row .n-input input");
    if (inputs.length > 0) (inputs[inputs.length - 1] as HTMLInputElement).focus();
  });
}

function removeKp(i: number) {
  formKps.value.splice(i, 1);
}

function openAsAdd() {
  editId.value = null;
  formName.value = "";
  formColor.value = PRESET_COLORS[Math.floor(Math.random() * PRESET_COLORS.length)];
  formKps.value = [];
  visible.value = true;
  nextTick(() => nameInputRef.value?.focus());
}

function openAsEdit(cls: ClassDefinition) {
  editId.value = cls.id;
  formName.value = cls.name;
  formColor.value = cls.color;
  const names = cls.keypoint_names ?? [];
  const colors = cls.keypoint_colors ?? [];
  formKps.value = names.map((n, i) => ({
    name: n,
    color: colors[i] ?? KP_DEFAULT_COLORS[i % KP_DEFAULT_COLORS.length],
  }));
  visible.value = true;
  nextTick(() => nameInputRef.value?.focus());
}

function reset() {
  editId.value = null;
  formName.value = "";
  formColor.value = "#FF6B6B";
  formKps.value = [];
}

const emit = defineEmits<{
  (e: "add", name: string, color: string, keypointNames?: string[], keypointColors?: string[]): void;
  (e: "update", id: number, name: string, color: string, keypointNames?: string[], keypointColors?: string[]): void;
  (e: "delete", id: number): void;
}>();

function handleConfirm() {
  const name = formName.value.trim();
  if (!name) return;
  const kps = formKps.value
    .map(kp => ({ name: kp.name.trim(), color: kp.color }))
    .filter(kp => kp.name.length > 0);
  const kpNames = kps.map(kp => kp.name);
  const kpColors = kps.map(kp => kp.color);
  if (editId.value !== null) {
    emit("update", editId.value, name, formColor.value, kpNames.length > 0 ? kpNames : undefined, kpColors.length > 0 ? kpColors : undefined);
  } else {
    emit("add", name, formColor.value, kpNames.length > 0 ? kpNames : undefined, kpColors.length > 0 ? kpColors : undefined);
  }
  visible.value = false;
}

function handleDelete() {
  if (editId.value !== null) {
    emit("delete", editId.value);
    visible.value = false;
  }
}

defineExpose({ openAsAdd, openAsEdit });
</script>

<style scoped>
.modal-body {
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
  color: var(--text-secondary);
  font-weight: 500;
}
.color-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.color-presets {
  display: flex;
  gap: 5px;
  flex-wrap: wrap;
  flex: 1;
}
.color-swatch {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid transparent;
  transition: border-color 0.1s, transform 0.1s;
}
.color-swatch:hover {
  transform: scale(1.15);
}
.color-swatch.active {
  border-color: #ffffff;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.4);
}
.color-picker-btn {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  border: 1px solid var(--border-subtle);
  overflow: hidden;
  transition: border-color 0.15s;
}
.color-picker-btn:hover {
  border-color: var(--accent);
}
.color-picker-btn input {
  position: absolute;
  top: -8px;
  left: -8px;
  width: calc(100% + 16px);
  height: calc(100% + 16px);
  opacity: 0;
  cursor: pointer;
}
.picker-icon {
  font-size: 14px;
  line-height: 1;
  filter: drop-shadow(0 0 2px rgba(0,0,0,0.5));
}
.color-preview {
  margin-top: 8px;
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 11px;
  color: #ffffff;
  font-weight: 500;
  display: inline-block;
  width: fit-content;
}

.del-btn {
  color: var(--danger) !important;
  font-size: 12px !important;
  padding: 0 8px !important;
}
.del-btn:hover {
  background: rgba(248, 113, 113, 0.12) !important;
}

.kp-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.kp-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.kp-idx {
  flex-shrink: 0;
  width: 18px;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-dim);
  text-align: center;
}
.kp-color-btn {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  cursor: pointer;
  position: relative;
  border: 2px solid rgba(255,255,255,0.3);
  overflow: hidden;
  transition: border-color 0.12s;
}
.kp-color-btn:hover {
  border-color: #fff;
}
.kp-color-btn input {
  position: absolute;
  width: 36px;
  height: 36px;
  top: -8px;
  left: -8px;
  opacity: 0;
  cursor: pointer;
}
.kp-row .n-input {
  flex: 1;
}
.modal-footer {
  display: flex;
  align-items: center;
  gap: 8px;
}
.spacer {
  flex: 1;
}
</style>