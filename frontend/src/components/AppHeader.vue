<template>
  <header class="app-header" ref="headerRef">
    <div class="header-left">
      <div class="logo-badge">E</div>
      <span class="header-title">EasyLabel</span>
    </div>
    <div class="header-center">
      <slot name="center" />
    </div>
    <div class="header-right">
      <button class="win-btn" @click="toggleTheme" :title="settingsStore.settings.theme_mode === 'light' ? '切换深色' : '切换浅色'" aria-label="切换主题">
        <svg v-if="settingsStore.settings.theme_mode === 'light'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
        <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
      </button>
      <button class="win-btn" @click="winMinimize" title="最小化">
        <svg width="10" height="10" viewBox="0 0 10 10"><rect y="4" width="10" height="1" fill="currentColor"/></svg>
      </button>
      <button class="win-btn" @click="winToggleMaximize" :title="isMaximized ? '还原' : '最大化'">
        <svg v-if="!isMaximized" width="10" height="10" viewBox="0 0 10 10"><rect x="0" y="0" width="10" height="10" stroke="currentColor" stroke-width="1" fill="none"/></svg>
        <svg v-else width="10" height="10" viewBox="0 0 10 10"><rect x="2" y="0" width="8" height="8" stroke="currentColor" stroke-width="1" fill="none"/><rect x="0" y="2" width="8" height="8" stroke="currentColor" stroke-width="1" fill="var(--bg-panel)"/></svg>
      </button>
      <button class="win-btn win-btn--close" @click="winClose" title="关闭">
        <svg width="10" height="10" viewBox="0 0 10 10"><line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" stroke-width="1.2"/><line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsStore } from "@/stores/settings";

const settingsStore = useSettingsStore();
const isMaximized = ref(false);
let unlisten: (() => void) | undefined;
const headerRef = ref<HTMLElement | null>(null);

function toggleTheme() {
  const modes = ["dark", "light", "system"] as const;
  const current = settingsStore.settings.theme_mode;
  const idx = modes.indexOf(current);
  settingsStore.settings.theme_mode = modes[(idx + 1) % modes.length];
}

onMounted(async () => {
  try {
    const w = getCurrentWindow();
    isMaximized.value = await w.isMaximized();
    unlisten = await w.onResized(async () => {
      isMaximized.value = await w.isMaximized();
    });
    const el = headerRef.value;
    if (el) {
      el.addEventListener("mousedown", async (e: MouseEvent) => {
        const target = e.target as HTMLElement;
        if (target.closest("button, a, input, select, textarea, [role='button']")) return;
        if (e.button !== 0) return;
        await w.startDragging();
      });
    }
  } catch { }
});

onBeforeUnmount(() => { unlisten?.(); });

async function winMinimize() { await getCurrentWindow().minimize(); }
async function winToggleMaximize() { await getCurrentWindow().toggleMaximize(); }
async function winClose() { await getCurrentWindow().close(); }
</script>

<style scoped>
.app-header {
  height: var(--topbar-height);
  background: var(--bg-panel);
  display: flex;
  align-items: center;
  padding: 0 8px;
  gap: 8px;
  flex-shrink: 0;
  user-select: none;
  cursor: default;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.logo-badge {
  width: 24px;
  height: 24px;
  border-radius: 5px;
  background: linear-gradient(145deg, #fb923c, var(--accent));
  color: #0a0a0a;
  font-size: 13px;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
  letter-spacing: -0.04em;
  flex-shrink: 0;
}
.header-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}
.header-center {
  flex: 1;
  display: flex;
  align-items: center;
  min-width: 0;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}
.win-btn {
  width: 36px;
  height: 32px;
  border-radius: 4px;
  background: transparent;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
  flex-shrink: 0;
}
.win-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.win-btn--close:hover {
  background: #c42b1c;
  color: #fff;
}
</style>