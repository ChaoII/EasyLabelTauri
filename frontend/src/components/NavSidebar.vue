<template>
  <aside class="nav-sidebar">
    <!-- Logo -->
    <div class="nav-logo-area">
      <div class="nav-logo-icon" @click="$emit('navigate', 'home')" title="首页">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
      </div>
    </div>

    <!-- 导航项目 -->
    <div class="nav-items">
      <div class="nav-item" :class="{ active: currentView === 'home' }" @click="$emit('navigate', 'home')" title="首页">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>
      </div>
      <div class="nav-item" :class="{ active: currentView === 'workspace' }" @click="$emit('navigate', 'workspace')" title="标注工作台">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
      </div>
    </div>

    <!-- 底部 -->
    <div class="nav-bottom">
      <div class="nav-item" @click="openSettings" title="设置">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 0 2.83 0 2 2 0 0 0 0-2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 0-2.83 0 2 2 0 0 0 0 2.83l.06.06A1.65 1.65 0 0 0 9 15.36a1.65 1.65 0 0 0 1-1.51V13a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09A1.65 1.65 0 0 0 19.32 15a1.65 1.65 0 0 0 1-1.51H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2v-.09A1.65 1.65 0 0 0 19.32 15z"/></svg>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { useSettingsStore } from "@/stores/settings";

defineProps<{ currentView: "home" | "workspace" }>();
defineEmits<{ (e: "navigate", view: "home" | "workspace"): void }>();

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

.nav-logo-area {
  width: 100%;
  display: flex;
  justify-content: center;
  padding: 14px 0 18px;
  border-bottom: 1px solid var(--border-subtle);
  margin-bottom: 12px;
}

.nav-logo-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  transition: all 0.15s;
}

.nav-logo-icon:hover {
  transform: scale(1.05);
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