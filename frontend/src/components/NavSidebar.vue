<template>
  <aside class="nav-sidebar" :class="{ collapsed: collapsed }">
    <div class="nav-top">
      <div class="nav-logo" :class="{ active: !projectStore.currentTaskId }" @click="projectStore.closeTask()" title="首页">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>
      </div>
      <div class="nav-logo" :class="{ active: !!projectStore.currentTaskId }" @click="goToAnnotation" title="标注工作台">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
      </div>
    </div>
    <div class="nav-bottom">
      <div class="nav-logo" @click="openSettings" title="设置">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 0 2.83 0 2 2 0 0 0 0-2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 0-2.83 0 2 2 0 0 0 0 2.83l.06.06A1.65 1.65 0 0 0 9 15.36a1.65 1.65 0 0 0 1-1.51V13a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09A1.65 1.65 0 0 0 19.32 15a1.65 1.65 0 0 0 1-1.51H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2v-.09A1.65 1.65 0 0 0 19.32 15z"/></svg>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { useProjectStore } from "@/stores/project";
import { useSettingsStore } from "@/stores/settings";

const projectStore = useProjectStore();
const settingsStore = useSettingsStore();

defineProps<{ collapsed?: boolean }>();

function goToAnnotation() {
  // 如果没有打开的任务，不做任何操作
}

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
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  flex-shrink: 0;
  z-index: 10;
}

.nav-top, .nav-bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.nav-logo {
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

.nav-logo:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-logo.active {
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
}

.nav-logo.active::before {
  content: '';
  position: absolute;
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  background: var(--accent);
  border-radius: 2px;
}
</style>