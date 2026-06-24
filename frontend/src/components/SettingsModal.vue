<template>
  <NModal
    v-model:show="visible"
    preset="card"
    title="设置"
    :mask-closable="true"
    style="width: 420px"
    :segmented="{ footer: true }"
  >
    <div class="settings-body">
      <!-- 外观 -->
      <div class="section">
        <div class="section-title">外观</div>

        <div class="field">
          <label class="field-label">强调色</label>
          <div class="color-row">
            <div
              v-for="c in settingsStore.ACCENT_PRESETS"
              :key="c"
              class="color-swatch"
              :class="{ active: settingsStore.settings.accent_color === c }"
              :style="{ background: c }"
              @click="settingsStore.setAccent(c)"
            />
          </div>
        </div>

        <div class="field">
          <label class="field-label">标注线宽</label>
          <div class="option-row">
            <div
              v-for="opt in lineWidthOptions"
              :key="opt.value"
              class="option-chip"
              :class="{ active: settingsStore.settings.annotation_line_width === opt.value }"
              @click="settingsStore.settings.annotation_line_width = opt.value"
            >
              {{ opt.label }}
            </div>
          </div>
        </div>
      </div>

      <!-- 标注 -->
      <div class="section">
        <div class="section-title">标注</div>

        <div class="field">
          <label class="field-label">默认标注颜色</label>
          <NInput
            v-model:value="settingsStore.settings.default_annotation_color"
            size="small"
            style="width: 120px; font-family: monospace"
            placeholder="#38bdf8"
          />
          <div class="color-preview-sm" :style="{ background: settingsStore.settings.default_annotation_color }" />
        </div>

        <div class="field-row">
          <label class="field-label">画布上显示标签</label>
          <NSwitch v-model:value="settingsStore.settings.show_labels" size="small" />
        </div>
      </div>

      <!-- 界面 -->
      <div class="section">
        <div class="section-title">界面</div>

        <div class="field-row">
          <label class="field-label">显示状态栏</label>
          <NSwitch v-model:value="settingsStore.settings.status_bar_visible" size="small" />
        </div>

        <div class="field-row">
          <label class="field-label">紧凑模式</label>
          <NSwitch v-model:value="settingsStore.settings.dense_mode" size="small" />
        </div>
      </div>
    </div>

    <template #footer>
      <div class="modal-footer">
        <NButton size="small" quaternary @click="handleReset">恢复默认</NButton>
        <div class="spacer" />
        <NButton size="small" @click="settingsStore.closeModal()">关闭</NButton>
      </div>
    </template>
  </NModal>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { NModal, NInput, NSwitch, NButton } from "naive-ui";
import { useSettingsStore } from "@/stores/settings";

const settingsStore = useSettingsStore();

const visible = computed({
  get: () => settingsStore.showModal,
  set: (v: boolean) => { if (!v) settingsStore.closeModal(); },
});

const lineWidthOptions = [
  { value: "thin", label: "细" },
  { value: "medium", label: "中" },
  { value: "thick", label: "粗" },
];

function handleReset() {
  settingsStore.reset();
}
</script>

<style scoped>
.settings-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-height: 60vh;
  overflow-y: auto;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding-bottom: 2px;
  border-bottom: 1px solid var(--border-subtle);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.field-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.color-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.color-swatch {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid transparent;
  transition: border-color 0.15s, transform 0.1s;
  flex-shrink: 0;
}

.color-swatch:hover {
  transform: scale(1.15);
}

.color-swatch.active {
  border-color: var(--text-primary);
  transform: scale(1.15);
}

.option-row {
  display: flex;
  gap: 6px;
}

.option-chip {
  padding: 4px 12px;
  border-radius: 4px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: background 0.1s, border-color 0.1s, color 0.1s;
}

.option-chip:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.option-chip.active {
  border-color: var(--accent);
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 15%, transparent);
}

.color-preview-sm {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  border: 1px solid var(--border-subtle);
  flex-shrink: 0;
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
