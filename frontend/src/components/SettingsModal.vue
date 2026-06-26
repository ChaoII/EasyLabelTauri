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

        <div class="field-row">
          <label class="field-label">主题</label>
          <div class="theme-toggle">
            <NButtonGroup size="small">
              <NButton :type="settingsStore.settings.theme_mode === 'light' ? 'primary' : 'default'" @click="settingsStore.settings.theme_mode = 'light'">
                <template #icon><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg></template>
                浅色
              </NButton>
              <NButton :type="settingsStore.settings.theme_mode === 'dark' ? 'primary' : 'default'" @click="settingsStore.settings.theme_mode = 'dark'">
                <template #icon><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg></template>
                深色
              </NButton>
              <NButton :type="settingsStore.settings.theme_mode === 'system' ? 'primary' : 'default'" @click="settingsStore.settings.theme_mode = 'system'">
                <template #icon><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><path d="M8 21h8M12 17v4"/></svg></template>
                跟随系统
              </NButton>
            </NButtonGroup>
          </div>
        </div>

        <div class="field">
          <label class="field-label">主题色</label>
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
import { NModal, NSwitch, NButton, NButtonGroup } from "naive-ui";
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
  overflow-x: visible;
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
  overflow: visible;
}

.color-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  flex: 1;
  overflow: visible;
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

.theme-toggle {
  display: flex;
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