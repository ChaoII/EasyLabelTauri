<template>
  <NEmpty v-if="classes.length === 0" description="暂无类别" size="small" />
  <div v-else class="class-list">
    <div
      v-for="cls in classes"
      :key="cls.id"
      class="class-item"
      :class="{
        active: multiMode ? selectedClassIds.includes(cls.id) : selectedClassId === cls.id,
        'has-annotation': isClassAnnotated(cls.id)
      }"
      @click="onClick(cls.id)"
      @dblclick.stop="emit('edit', cls)"
      :title="`${multiMode ? '点击切换' : '单击选择，'}双击编辑「${cls.name}」`"
    >
      <div
        v-if="multiMode"
        class="class-check"
        :class="{ checked: selectedClassIds.includes(cls.id) }"
        :style="{ '--cls-color': cls.color }"
      >
        <svg v-if="selectedClassIds.includes(cls.id)" width="8" height="8" viewBox="0 0 8 8" fill="none">
          <path d="M1.5 4L3.2 6L6.5 2" stroke="white" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <div class="color-dot" :style="{ background: cls.color }"></div>
      <span class="name">{{ cls.name }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { NEmpty } from "naive-ui";
import type { Annotation, ClassDefinition } from "@/utils/types";

const props = defineProps<{
  classes: ClassDefinition[];
  selectedClassId: number;
  annotations?: Annotation[];
  /** 多选模式（分类工具激活时） */
  multiMode?: boolean;
  selectedClassIds?: number[];
}>();

const emit = defineEmits<{
  (e: "select", id: number): void;
  (e: "toggle", id: number): void;
  (e: "edit", cls: ClassDefinition): void;
}>();

const selectedClassIds = computed(() => props.selectedClassIds ?? []);

function isClassAnnotated(classId: number): boolean {
  if (!props.annotations || props.annotations.length === 0) return false;
  return props.annotations.some((ann) => {
    if ("class_id" in ann) return ann.class_id === classId;
    if ("class_ids" in ann && Array.isArray(ann.class_ids))
      return ann.class_ids.includes(classId);
    return false;
  });
}

function onClick(id: number) {
  if (props.multiMode) {
    emit("toggle", id);
  } else {
    emit("select", id);
  }
}
</script>

<style scoped>
.class-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.class-item {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 5px 8px;
  border-radius: var(--radius);
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s;
  position: relative;
  overflow: hidden;
  border: 1px solid transparent;
}

.class-item:hover {
  background: var(--bg-hover);
}

.class-item.active {
  background: rgba(249, 115, 22, 0.12);
  border-left: 3px solid var(--accent);
  padding-left: 6px;
}

.class-item.has-annotation .color-dot {
  box-shadow: 0 0 0 2px #22c55e;
}

.class-check {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  border-radius: 4px;
  border: 1.5px solid var(--border-subtle);
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.12s, border-color 0.12s;
}

.class-check.checked {
  background: var(--cls-color, var(--accent));
  border-color: var(--cls-color, var(--accent));
}

.color-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--cls-color, #fff);
  transition: box-shadow 0.15s;
}

.name {
  font-size: 12px;
  color: var(--text-primary);
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
