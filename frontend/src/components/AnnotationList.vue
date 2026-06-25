<template>
  <NEmpty v-if="annotations.length === 0" description="暂无标注" size="small" />
  <div v-else class="annotation-list">
    <div
      v-for="ann in annotations"
      :key="ann.id"
      class="ann-item"
      :class="{ active: ann.id === selectedId }"
      @click="emit('select', ann.id)"
    >
      <div class="ann-icon" aria-hidden="true">
        <Square v-if="ann.type === 'AxisAlignedBox'" :size="14" class="ann-icon-svg" />
        <Diamond v-else-if="ann.type === 'RotatedBox'" :size="14" class="ann-icon-svg" />
        <span v-else>{{ typeIcon(ann.type) }}</span>
      </div>
      <div class="ann-info">
        <div class="ann-type">{{ typeName(ann.type) }}</div>
        <div class="ann-class" :style="{ color: getClassColor(annClassId(ann)) }">
          <template v-if="ann.type === 'Classification' && 'class_ids' in ann && Array.isArray(ann.class_ids)">
            <span v-for="(cid, ci) in ann.class_ids" :key="ci" class="cls-tag" :style="{ color: getClassColor(cid), background: getClassColor(cid) + '22' }">
              {{ getClassName(cid) }}
            </span>
          </template>
          <template v-else>
            {{ getClassName(annClassId(ann)) }}
          </template>
        </div>
        <div v-if="ann.type === 'Keypoint'" class="ann-kp-info">
          {{ keypointCountInfo(ann) }}
        </div>
      </div>
      <NButton
        quaternary
        circle
        size="tiny"
        class="del-btn"
        @click.stop="emit('delete', ann.id)"
      >
        <template #icon>
          <X />
        </template>
      </NButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NButton, NEmpty } from "naive-ui";
import { Diamond, Square, X } from "lucide-vue-next";
import type { Annotation, ClassDefinition, KeypointAnnotation } from "@/utils/types";

const props = defineProps<{
  annotations: Annotation[];
  selectedId: string | null;
  classes: ClassDefinition[];
}>();

const emit = defineEmits<{
  (e: "select", id: string): void;
  (e: "delete", id: string): void;
}>();

function typeIcon(type: Annotation["type"]): string {
  const map: Record<string, string> = {
    AxisAlignedBox: "□",
    RotatedBox: "◇",
    Polygon: "⬡",
    Keypoint: "✦",
    Ocr: "T",
    Classification: "☰",
  };
  return map[type] ?? "?";
}

function typeName(type: Annotation["type"]): string {
  const map: Record<string, string> = {
    AxisAlignedBox: "矩形",
    RotatedBox: "旋转框",
    Polygon: "多边形",
    Keypoint: "关键点",
    Ocr: "OCR",
    Classification: "分类",
  };
  return map[type] ?? type;
}

function getClassColor(id: number): string {
  return props.classes[id]?.color ?? "#fff";
}

function getClassName(id: number): string {
  return props.classes[id]?.name ?? `Class ${id}`;
}

function annClassId(ann: Annotation): number {
  if ("class_id" in ann) return ann.class_id;
  if ("class_ids" in ann && Array.isArray(ann.class_ids) && ann.class_ids.length > 0) return ann.class_ids[0];
  return 0;
}

function keypointCountInfo(ann: Annotation): string {
  if (ann.type !== "Keypoint") return "";
  const visible = (ann as KeypointAnnotation).keypoints.filter((k) => k.visibility === 2).length;
  const total = (ann as KeypointAnnotation).keypoints.length;
  return `关键点 ${visible}/${total}`;
}
</script>

<style scoped>
.annotation-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ann-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: var(--radius);
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s;
  position: relative;
  overflow: hidden;
  border: 1px solid transparent;
}

.ann-item:hover {
  background: var(--bg-hover);
}

.ann-item.active {
  background: rgba(249, 115, 22, 0.12);
  border-left: 3px solid var(--accent);
  padding-left: 5px;
}

.ann-icon {
  font-size: 14px;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--text-secondary);
}

.ann-icon-svg {
  flex-shrink: 0;
}

.ann-info {
  flex: 1;
  min-width: 0;
}

.ann-type {
  font-size: 11px;
  color: var(--text-primary);
}

.ann-class {
  font-size: 10px;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cls-tag {
  display: inline-block;
  padding: 1px 5px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 500;
  line-height: 1.4;
}

.ann-kp-info {
  font-size: 10px;
  color: var(--text-dim);
  margin-top: 2px;
}

.del-btn {
  width: 24px;
  height: 24px;
  min-width: 24px;
  flex-shrink: 0;
  color: var(--text-dim);
}

.del-btn:hover {
  color: var(--danger) !important;
}
</style>
