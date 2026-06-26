<template>
  <header class="app-header" ref="headerRef">
    <!-- 左侧：Logo + 标题 -->
    <div class="header-left">
      <div class="logo-badge">E</div>
      <span class="header-title">EasyLabel</span>
      <div class="sep"></div>
    </div>

    <!-- 中间：插槽（页面标题等） -->
    <div class="header-center">
      <slot name="center" />
    </div>

    <!-- 右侧：设置 + 窗口控制 -->
    <div class="header-right">
      <!-- 设置按钮 -->
      <button class="win-btn" @click="openSettings" title="设置" aria-label="设置">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </button>

      <!-- 最小化 -->
      <button class="win-btn" @click="winMinimize" title="最小化" aria-label="最小化">
        <svg width="10" height="10" viewBox="0 0 10 10"><rect y="4" width="10" height="1" fill="currentColor"/></svg>
      </button>
      <!-- 最大化 / 还原 -->
      <button class="win-btn" @click="winToggleMaximize" :title="isMaximized ? '还原' : '最大化'" :aria-label="isMaximized ? '还原' : '最大化'">
        <svg v-if="!isMaximized" width="10" height="10" viewBox="0 0 10 10">
          <rect x="0" y="0" width="10" height="10" stroke="currentColor" stroke-width="1" fill="none"/>
        </svg>
        <svg v-else width="10" height="10" viewBox="0 0 10 10">
          <rect x="2" y="0" width="8" height="8" stroke="currentColor" stroke-width="1" fill="none"/>
          <rect x="0" y="2" width="8" height="8" stroke="currentColor" stroke-width="1" fill="var(--bg-panel)"/>
        </svg>
      </button>
      <!-- 关闭 -->
      <button class="win-btn win-btn--close" @click="winClose" title="关闭" aria-label="关闭">
        <svg width="10" height="10" viewBox="0 0 10 10">
          <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" stroke-width="1.2"/>
          <line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" stroke-width="1.2"/>
        </svg>
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

onMounted(async () => {
  try {
    const w = getCurrentWindow();
    isMaximized.value = await w.isMaximized();
    unlisten = await w.onResized(async () => {
      isMaximized.value = await w.isMaximized();
    });

    // 手动处理窗口拖动（data-tauri-drag-region 在 Tauri2 中不够可靠）
    const el = headerRef.value;
    if (el) {
      el.addEventListener("mousedown", async (e: MouseEvent) => {
        // 仅在非按钮等交互元素上触发拖动
        const target = e.target as HTMLElement;
        if (target.closest("button, a, input, select, textarea, [role='button']")) return;
        if (e.button !== 0) return;
        await w.startDragging();
      });
    }
  } catch { /* 非 Tauri 环境忽略 */ }
});

onBeforeUnmount(() => { unlisten?.(); });

async function winMinimize() { await getCurrentWindow().minimize(); }
async function winToggleMaximize() { await getCurrentWindow().toggleMaximize(); }
async function winClose() { await getCurrentWindow().close(); }

function openSettings() {
  settingsStore.openModal();
}
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
  /* cursor 必须在这里设，保证鼠标移入 header 任意空白处都显示拖动手势 */
  cursor: default;
}

/* ---- 左侧 ---- */
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

.sep {
  width: 1px;
  height: 20px;
  background: var(--border-subtle);
  margin: 0 2px;
}

/* ---- 中间插槽 ---- */
.header-center {
  flex: 1;
  display: flex;
  align-items: center;
  min-width: 0;
}

/* ---- 右侧：窗口按钮 ---- */
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
