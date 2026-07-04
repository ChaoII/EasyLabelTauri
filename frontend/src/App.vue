<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="themeOverrides" :locale="zhCN">
    <NMessageProvider>
      <div class="app-layout">
        <AppHeader>
          <template #center>
            <template v-if="projectStore.currentTask">
              <div class="ann-title">
                <NButton class="back-btn" @click="onBackFromAnnotation" title="返回任务列表" quaternary circle size="tiny">
                  <template #icon><ChevronLeft :size="16" /></template>
                </NButton>
                <div class="task-type-badge" :style="{ background: typeColor() + '22', color: typeColor() }">
                  {{ TASK_TYPE_ICONS[projectStore.currentTask.task_type] }}
                </div>
                <span class="task-name">{{ projectStore.currentTask.name }}</span>
              </div>
              <div class="header-progress" v-if="currentTaskTotal > 0">
                <span class="progress-text">{{ currentTaskAnnotated }} / {{ currentTaskTotal }}</span>
                <div class="progress-mini">
                  <div class="progress-mini-fill" :class="{ 'fill-done': progressPct >= 100 }" :style="{ width: progressPct + '%' }" />
                </div>
              </div>
            </template>
            <span v-else-if="currentView === 'home'" style="font-size:14px;font-weight:600;">统计概览</span>
            <span v-else-if="currentView === 'workspace'" style="font-size:14px;font-weight:600;">标注工作台</span>
            <span v-else-if="currentView === 'models'" style="font-size:14px;font-weight:600;">模型管理</span>
          </template>
        </AppHeader>
        <div class="app-body">
          <NavSidebar :current-view="currentView" @navigate="onNavigate" />
          <main class="app-main">
            <AnnotationView
              v-if="projectStore.currentTaskId"
              :task-id="projectStore.currentTaskId"
              @back="onBackFromAnnotation"
            />
            <HomeDashboard v-else-if="currentView === 'home'" />
            <ModelManage v-else-if="currentView === 'models'" />
            <HomeView v-else-if="currentView === 'workspace'" />
          </main>
        </div>
      </div>
    </NMessageProvider>
    <SettingsModal />
  </NConfigProvider>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, watch } from "vue";
import { darkTheme, lightTheme, zhCN } from "naive-ui";
import { NConfigProvider, NMessageProvider } from "naive-ui";
import { useProjectStore } from "@/stores/project";
import { useSettingsStore } from "@/stores/settings";
import { useModelStore } from "@/stores/models";
import HomeView from "@/components/HomeView.vue";
import AnnotationView from "@/components/AnnotationView.vue";
import SettingsModal from "@/components/SettingsModal.vue";
import NavSidebar from "@/components/NavSidebar.vue";
import HomeDashboard from "@/components/HomeDashboard.vue";
import ModelManage from "@/components/ModelManage.vue";
import AppHeader from "@/components/AppHeader.vue";
import { ChevronLeft } from "lucide-vue-next";
import { TASK_TYPE_ICONS, type TaskType } from "@/utils/taskTypes";
import { NButton } from "naive-ui";

const projectStore = useProjectStore();
const settingsStore = useSettingsStore();
const modelStore = useModelStore();

const currentView = ref<"home" | "workspace" | "models">("home");

function typeColor(): string {
  const m: Record<string, string> = {
    classification: "#6366f1", detection: "#3b82f6", rotated_detection: "#8b5cf6",
    keypoint: "#eab308", segmentation: "#22c55e", ocr: "#06b6d4",
  };
  return m[projectStore.currentTask?.task_type ?? ""] ?? "#6b7280";
}
const currentTaskTotal = computed(() => projectStore.currentTask?.stats?.total_images ?? 0);
const currentTaskAnnotated = computed(() => projectStore.currentTask?.stats?.annotated_images ?? 0);
const progressPct = computed(() =>
  currentTaskTotal.value === 0 ? 0 : Math.round(currentTaskAnnotated.value / currentTaskTotal.value * 100)
);

function onNavigate(view: "home" | "workspace" | "models") {
  currentView.value = view;
  if (view === "workspace" && !projectStore.currentTaskId) {
    // 进入工作台但无打开任务
  }
}

function onBackFromAnnotation() {
  projectStore.closeTask();
  // 从标注返回后留在工作台
}

const accent = computed(() => settingsStore.settings.accent_color);
const themeMode = computed(() => settingsStore.settings.theme_mode);
const dense = computed(() => settingsStore.settings.dense_mode);

function effectiveTheme(): "dark" | "light" {
  const mode = themeMode.value;
  if (mode === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return mode;
}

const naiveTheme = computed(() => effectiveTheme() === "light" ? lightTheme : darkTheme);

function applyTheme() {
  const isLight = effectiveTheme() === "light";
  document.documentElement.setAttribute("data-theme", isLight ? "light" : "dark");
  if (isLight) {
    document.documentElement.style.setProperty("--bg-app", "#f5f5f5");
    document.documentElement.style.setProperty("--bg-panel", "#ffffff");
    document.documentElement.style.setProperty("--bg-elevated", "#e8e8ec");
    document.documentElement.style.setProperty("--bg-hover", "#dddde2");
    document.documentElement.style.setProperty("--bg-canvas", "#e8e8ec");
    document.documentElement.style.setProperty("--border-subtle", "#d4d4d8");
    document.documentElement.style.setProperty("--text-primary", "#18181b");
    document.documentElement.style.setProperty("--text-secondary", "#52525b");
    document.documentElement.style.setProperty("--text-dim", "#a1a1aa");
    document.documentElement.style.setProperty("--leftbar-bg", "#e0e0e0");
    document.documentElement.style.setProperty("--leftbar-border", "#d0d0d0");
    document.documentElement.style.setProperty("--bg-app-rgb", "245,245,245");
  } else {
    document.documentElement.style.setProperty("--bg-app", "#1e1e1e");
    document.documentElement.style.setProperty("--bg-panel", "#252526");
    document.documentElement.style.setProperty("--bg-elevated", "#2d2d30");
    document.documentElement.style.setProperty("--bg-hover", "#37373d");
    document.documentElement.style.setProperty("--bg-canvas", "#2a2a2a");
    document.documentElement.style.setProperty("--border-subtle", "#3e3e42");
    document.documentElement.style.setProperty("--text-primary", "#e4e4e7");
    document.documentElement.style.setProperty("--text-secondary", "#a1a1aa");
    document.documentElement.style.setProperty("--text-dim", "#71717a");
    document.documentElement.style.setProperty("--leftbar-bg", "#0a0a0a");
    document.documentElement.style.setProperty("--leftbar-border", "#1f1f1f");
    document.documentElement.style.setProperty("--bg-app-rgb", "30,30,30");
  }
  applyAccentColor();
}

function applyDense(v: boolean) {
  document.documentElement.style.setProperty("--topbar-height", v ? "38px" : "48px");
  document.documentElement.style.setProperty("--leftbar-width", v ? "48px" : "62px");
  document.documentElement.style.setProperty("--statusbar-height", v ? "24px" : "28px");
}

let systemThemeMq: MediaQueryList | null = null;

onMounted(async () => {
  await projectStore.loadProject();
  await settingsStore.load();
  await modelStore.load();
  applyTheme();
  applyDense(settingsStore.settings.dense_mode);
  // 监听系统主题变化
  systemThemeMq = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeMq.addEventListener("change", applyTheme);
});

onBeforeUnmount(() => {
  systemThemeMq?.removeEventListener("change", applyTheme);
});

watch(accent, applyAccentColor);
watch(dense, applyDense);
watch(themeMode, applyTheme);

function lighten(hex: string, amount: number) {
  const num = parseInt(hex.replace("#", ""), 16);
  const r = Math.min(255, ((num >> 16) & 0xff) + amount);
  const g = Math.min(255, ((num >> 8) & 0xff) + amount);
  const b = Math.min(255, (num & 0xff) + amount);
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
}

function darken(hex: string, amount: number) {
  const num = parseInt(hex.replace("#", ""), 16);
  const r = Math.max(0, ((num >> 16) & 0xff) - amount);
  const g = Math.max(0, ((num >> 8) & 0xff) - amount);
  const b = Math.max(0, (num & 0xff) - amount);
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
}

function applyAccentColor() {
  document.documentElement.style.setProperty("--accent", accent.value);
  document.documentElement.style.setProperty("--accent-dim", darken(accent.value, 70));
}

const themeOverrides = computed(() => {
  const isLight = effectiveTheme() === "light";
  return {
    common: {
      borderRadius: "6px",
      primaryColor: accent.value,
      primaryColorHover: lighten(accent.value, 20),
      primaryColorPressed: darken(accent.value, 40),
      bodyColor: isLight ? "#f5f5f5" : "#1e1e1e",
      cardColor: isLight ? "#ffffff" : "#252526",
      modalColor: isLight ? "#ffffff" : "#252526",
      popoverColor: isLight ? "#ffffff" : "#252526",
      tableColor: isLight ? "#ffffff" : "#252526",
      inputColor: isLight ? "#e8e8ec" : "#2d2d30",
      actionColor: isLight ? "#e8e8ec" : "#2d2d30",
      hoverColor: isLight ? "#dddde2" : "#37373d",
      borderColor: isLight ? "#d4d4d8" : "#3e3e42",
      dividerColor: isLight ? "#d4d4d8" : "#3e3e42",
      textColorBase: isLight ? "#18181b" : "#e4e4e7",
      textColor1: isLight ? "#18181b" : "#e4e4e7",
      textColor2: isLight ? "#52525b" : "#a1a1aa",
      textColor3: isLight ? "#a1a1aa" : "#71717a",
      textColorDisabled: isLight ? "#c0c0c4" : "#52525b",
      fontFamily: "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif",
      fontSize: "13px",
    },
    Button: {
      quaternaryColor: "transparent",
      quaternaryColorHover: isLight ? "#dddde2" : "#37373d",
      quaternaryColorPressed: isLight ? "#d4d4d8" : "#3e3e42",
      quaternaryColorDisabled: "transparent",
      textColorQuaternary: isLight ? "#18181b" : "#e4e4e7",
      textColorQuaternaryHover: isLight ? "#000" : "#fff",
      textColorQuaternaryPressed: isLight ? "#52525b" : "#a1a1aa",
      textColorQuaternaryDisabled: isLight ? "#c0c0c4" : "#52525b",
      borderRadiusMedium: "6px",
    },
    Tag: { borderRadius: "4px" },
    Scrollbar: { color: isLight ? "#d4d4d8" : "#3e3e42", colorHover: isLight ? "#a1a1aa" : "#71717a" },
    Card: { borderRadius: "6px", borderColor: isLight ? "#d4d4d8" : "#3e3e42", color: isLight ? "#ffffff" : "#252526" },
    Divider: { color: isLight ? "#d4d4d8" : "#3e3e42" },
    Badge: { color: accent.value },
    Drawer: { color: isLight ? "#f5f5f5" : "#1e1e1e" },
  };
});
</script>

<style>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}
.app-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.app-main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>