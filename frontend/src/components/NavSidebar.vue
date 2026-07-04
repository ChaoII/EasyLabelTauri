<template>
  <aside class="nav-sidebar">
    <!-- 导航项目 -->
    <div class="nav-items">
      <NTooltip placement="right" :delay="400">
        <template #trigger>
          <div class="nav-item" :class="{ active: currentView === 'home' }" @click="$emit('navigate', 'home')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>
          </div>
        </template>
        首页
      </NTooltip>
      <NTooltip placement="right" :delay="400">
        <template #trigger>
          <div class="nav-item" :class="{ active: currentView === 'workspace' }" @click="$emit('navigate', 'workspace')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
          </div>
        </template>
        标注工作台
      </NTooltip>
      <NTooltip placement="right" :delay="400">
        <template #trigger>
          <div class="nav-item" :class="{ active: currentView === 'models' }" @click="$emit('navigate', 'models')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
          </div>
        </template>
        模型管理
      </NTooltip>
    </div>

    <!-- 底部 -->
    <div class="nav-bottom">
      <NTooltip placement="right" :delay="400">
        <template #trigger>
          <div class="nav-item" @click="openSettings">
            <Settings :size="20" />
          </div>
        </template>
        设置
      </NTooltip>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { NTooltip } from "naive-ui";
import { useSettingsStore } from "@/stores/settings";
import { Settings } from "lucide-vue-next";

defineProps<{ currentView: "home" | "workspace" | "models" }>();
defineEmits<{ (e: "navigate", view: "home" | "workspace" | "models"): void }>();

const settingsStore = useSettingsStore();

function openSettings() {
  settingsStore.openModal();
}
</script>

<style scoped>
.nav-sidebar {
  width: 62px;
  height: 100%;
  background: var(--bg-panel);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0;
  flex-shrink: 0;
  z-index: 10;
  overflow: hidden;
}

.nav-items {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  flex: 1;
}

.nav-bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding-bottom: 12px;
}

.nav-item {
  width: 42px;
  height: 42px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.15s;
  position: relative;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
}

.nav-item.active::before {
  content: '';
  position: absolute;
  left: -10px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  background: var(--accent);
  border-radius: 2px;
}
</style>