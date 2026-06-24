<template>
  <NTooltip trigger="hover" :delay="400" placement="right">
    <template #trigger>
      <NButton
        quaternary
        size="small"
        block
        class="tool-row"
        :class="{
          'tool-active': modelValue === tool.name,
          'tool-row--icon-only': hideLabel,
        }"
        @click="$emit('update:modelValue', tool.name)"
      >
        <span class="tool-inner">
          <span class="tool-icon">
            <slot />
          </span>
          <span v-if="!hideLabel" class="tool-label">{{ tool.label }}</span>
        </span>
      </NButton>
    </template>
    {{ tool.tip }}
  </NTooltip>
</template>

<script setup lang="ts">
import { NButton, NTooltip } from "naive-ui";
import type { ToolName } from "@/utils/types";

withDefaults(
  defineProps<{
    modelValue: ToolName;
    tool: { name: ToolName; tip: string; label: string };
    hideLabel?: boolean;
  }>(),
  { hideLabel: false }
);
defineEmits<{
  (e: "update:modelValue", v: ToolName): void;
}>();
</script>

<style scoped>
.tool-row {
  justify-content: center !important;
  height: auto !important;
  min-height: 36px;
  padding: 5px 3px !important;
  border-radius: 6px !important;
  border: 1px solid transparent !important;
  font-weight: 400;
  transition: background 0.12s, border-color 0.12s;
}

/* 仅图标：收窄内边距、整体居中，配合窄侧栏 */
.tool-row--icon-only {
  justify-content: center !important;
  min-height: 32px;
  padding: 3px 2px !important;
}

.tool-row--icon-only .tool-inner {
  flex-direction: column;
  gap: 0;
  width: auto;
}

.tool-row:hover {
  background: rgba(255, 255, 255, 0.05) !important;
}

.tool-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  width: auto;
  max-width: 100%;
  min-width: 0;
  margin-inline: auto;
}

.tool-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  color: var(--text-secondary);
  transition: color 0.12s;
}

.tool-icon :deep(svg) {
  width: 18px;
  height: 18px;
  opacity: 0.8;
  transition: opacity 0.12s;
}

.tool-label {
  font-size: 11px;
  line-height: 1.25;
  color: var(--text-dim);
  text-align: center;
  white-space: normal;
  word-break: keep-all;
  overflow: visible;
  max-width: 100%;
  width: 100%;
  transition: color 0.12s;
}

/* 选中：左侧细条 + 略提亮 */
.tool-active {
  background: rgba(255, 255, 255, 0.06) !important;
  border-color: rgba(255, 255, 255, 0.08) !important;
  box-shadow: inset 2px 0 0 var(--accent);
}

.tool-active .tool-icon,
.tool-active .tool-label {
  color: var(--text-primary);
}

.tool-active .tool-icon :deep(svg) {
  opacity: 1;
  color: var(--text-primary);
}
</style>
