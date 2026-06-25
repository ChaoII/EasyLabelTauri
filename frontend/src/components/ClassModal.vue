<template>
  <NModal
    v-model:show="visible"
    preset="card"
    :title="editId === null ? '添加类别' : '编辑类别'"
    :mask-closable="true"
    style="width: 360px"
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
          <NInput
            v-model:value="formColor"
            size="small"
            style="width: 90px; flex-shrink: 0; font-family: monospace"
            placeholder="#ff0000"
          />
        </div>
        <!-- 预览 -->
        <div class="color-preview" :style="{ background: formColor }">
          {{ formName || "预览" }}
        </div>
      </div>

      <!-- 关键点名称（仅关键点任务显示） -->
      <div v-if="taskType === 'keypoint'" class="field">
        <label class="field-label">
          关键点名称
          <span class="field-hint">（用于关键点工具，逗号分隔，示例: nose,left_eye,right_eye）</span>
        </label>
        <NInput
          v-model:value="formKeypointNames"
          placeholder="例如: top_left, top_right, bottom_right, bottom_left"
          size="small"
          :maxlength="200"
        />
        <div class="kp-preview" v-if="parsedKeypointNames.length > 0">
          <span class="kp-chip" v-for="(name, i) in parsedKeypointNames" :key="i">
            {{ i + 1 }}. {{ name }}
          </span>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="modal-footer">
        <NButton
          v-if="editId !== null"
          size="small"
          type="error"
          quaternary
          @click="handleDelete"
        >
          删除
        </NButton>
        <div class="spacer" />
        <NButton size="small" @click="visible = false">取消</NButton>
        <NButton size="small" type="primary" @click="handleConfirm">确认</NButton>
      </div>
    </template>
  </NModal>
</template>

<script setup lang="ts">
import { ref, nextTick, computed } from "vue";
import { NModal, NInput, NButton } from "naive-ui";
import type { ClassDefinition } from "@/utils/types";
import type { TaskType } from "@/utils/taskTypes";

const props = defineProps<{
  taskType?: TaskType;
}>();

const PRESET_COLORS = [
  "#FF6B6B", "#FF9F43", "#FFE66D", "#78E08F",
  "#4ECDC4", "#82CCDD", "#AA96DA", "#D980FA",
  "#F38181", "#95E1D3", "#A8D8EA", "#FCBAD3",
];

const visible = ref(false);
const editId = ref<number | null>(null);
const formName = ref("");
const formColor = ref("#FF6B6B");
const formKeypointNames = ref("");

const parsedKeypointNames = computed(() => {
  return formKeypointNames.value
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
});

const nameInputRef = ref<InstanceType<typeof NInput> | null>(null);

function openAsAdd() {
  editId.value = null;
  formName.value = "";
  formColor.value = PRESET_COLORS[Math.floor(Math.random() * PRESET_COLORS.length)];
  visible.value = true;
  nextTick(() => nameInputRef.value?.focus());
}

function openAsEdit(cls: ClassDefinition) {
  editId.value = cls.id;
  formName.value = cls.name;
  formColor.value = cls.color;
  formKeypointNames.value = cls.keypoint_names?.join(", ") ?? "";
  visible.value = true;
  nextTick(() => nameInputRef.value?.focus());
}

function reset() {
  editId.value = null;
  formName.value = "";
  formColor.value = "#FF6B6B";
  formKeypointNames.value = "";
}

const emit = defineEmits<{
  (e: "add", name: string, color: string, keypointNames?: string[]): void;
  (e: "update", id: number, name: string, color: string, keypointNames?: string[]): void;
  (e: "delete", id: number): void;
}>();

function handleConfirm() {
  const name = formName.value.trim();
  if (!name) return;
  if (editId.value !== null) {
    emit("update", editId.value, name, formColor.value, parsedKeypointNames.value);
  } else {
    emit("add", name, formColor.value, parsedKeypointNames.value);
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
.modal-footer {
  display: flex;
  align-items: center;
  gap: 8px;
}
.spacer {
  flex: 1;
}

.field-hint {
  font-size: 10px;
  color: var(--text-dim);
  font-weight: 400;
  margin-left: 4px;
}

.kp-preview {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 6px;
}

.kp-chip {
  padding: 2px 8px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: 4px;
  font-size: 10px;
  color: var(--text-secondary);
}
</style>
