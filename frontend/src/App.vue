<template>
  <NConfigProvider :theme="darkTheme" :theme-overrides="themeOverrides">
    <NMessageProvider>
      <!-- 有任务时显示标注页，否则显示首页 -->
      <AnnotationView
        v-if="projectStore.currentTaskId"
        :task-id="projectStore.currentTaskId"
        @back="projectStore.closeTask()"
      />
      <HomeView v-else />
    </NMessageProvider>
    <SettingsModal />
  </NConfigProvider>
</template>

<script setup lang="ts">
import { onMounted, computed, watch } from "vue";
import { darkTheme } from "naive-ui";
import { NConfigProvider, NMessageProvider } from "naive-ui";
import { useProjectStore } from "@/stores/project";
import { useSettingsStore } from "@/stores/settings";
import HomeView from "@/components/HomeView.vue";
import AnnotationView from "@/components/AnnotationView.vue";
import SettingsModal from "@/components/SettingsModal.vue";

const projectStore = useProjectStore();
const settingsStore = useSettingsStore();

const accent = computed(() => settingsStore.settings.accent_color);
const dense = computed(() => settingsStore.settings.dense_mode);

function applyDense(v: boolean) {
  document.documentElement.style.setProperty("--topbar-height", v ? "38px" : "48px");
  document.documentElement.style.setProperty("--leftbar-width", v ? "48px" : "62px");
  document.documentElement.style.setProperty("--statusbar-height", v ? "24px" : "28px");
}

onMounted(async () => {
  await projectStore.loadProject();
  await settingsStore.load();
  applyAccentColor();
  applyDense(settingsStore.settings.dense_mode);
});

watch(accent, applyAccentColor);
watch(dense, applyDense);

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

// Naive UI theme overrides (dynamic accent)
const themeOverrides = computed(() => ({
  common: {
    borderRadius: "6px",
    primaryColor: accent.value,
    primaryColorHover: lighten(accent.value, 20),
    primaryColorPressed: darken(accent.value, 40),
    bodyColor: "#1e1e1e",
    cardColor: "#252526",
    modalColor: "#252526",
    popoverColor: "#252526",
    tableColor: "#252526",
    inputColor: "#2d2d30",
    actionColor: "#2d2d30",
    hoverColor: "#37373d",
    borderColor: "#3e3e42",
    dividerColor: "#3e3e42",
    textColorBase: "#e4e4e7",
    textColor1: "#e4e4e7",
    textColor2: "#a1a1aa",
    textColor3: "#71717a",
    textColorDisabled: "#52525b",
    fontFamily: "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif",
    fontSize: "13px",
  },
  Button: {
    quaternaryColor: "transparent",
    quaternaryColorHover: "#37373d",
    quaternaryColorPressed: "#3e3e42",
    quaternaryColorDisabled: "transparent",
    textColorQuaternary: "#e4e4e7",
    textColorQuaternaryHover: "#fff",
    textColorQuaternaryPressed: "#a1a1aa",
    textColorQuaternaryDisabled: "#52525b",
    borderRadiusMedium: "6px",
  },
  Tag: { borderRadius: "4px" },
  Scrollbar: { color: "#3e3e42", colorHover: "#71717a" },
  Card: { borderRadius: "6px", borderColor: "#3e3e42", color: "#252526" },
  Divider: { color: "#3e3e42" },
  Badge: { color: accent.value },
  Drawer: { color: "#1e1e1e" },
}));
</script>
