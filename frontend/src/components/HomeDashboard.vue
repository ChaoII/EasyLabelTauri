<template>
  <div class="dashboard">
    <!-- 统计卡片 -->
    <div class="stat-cards">
      <div class="stat-card" v-for="s in stats" :key="s.label">
        <div class="stat-icon" :style="{ color: s.color, background: s.color + '18' }">{{ s.icon }}</div>
        <div class="stat-body">
          <div class="stat-value">{{ s.value }}</div>
          <div class="stat-label">{{ s.label }}</div>
        </div>
        <div class="stat-change" :style="{ color: s.change > 0 ? '#22c55e' : '#f87171' }">
          {{ s.changeText }}
        </div>
      </div>
    </div>

    <!-- 图表区 -->
    <div class="chart-grid">
      <div class="chart-card">
        <div class="chart-title">各类别标注数量</div>
        <v-chart :option="barOption" autoresize style="width:100%;height:220px" />
      </div>
      <div class="chart-card">
        <div class="chart-title">标注进度</div>
        <v-chart :option="donutOption" autoresize style="width:100%;height:220px" />
      </div>
      <div class="chart-card">
        <div class="chart-title">各类别图片分布</div>
        <v-chart :option="imgBarOption" autoresize style="width:100%;height:220px" />
      </div>
      <div class="chart-card">
        <div class="chart-title">最近编辑</div>
        <div class="recent-list">
          <div v-for="t in recentTasks" :key="t.id" class="recent-item" @click="openTask(t.id)">
            <span class="recent-type" :style="{ color: typeColors[t.task_type] }">{{ TASK_TYPE_ICONS[t.task_type] }}</span>
            <span class="recent-name">{{ t.name }}</span>
            <span class="recent-time">{{ formatTime(t.updated_at) }}</span>
          </div>
          <div v-if="recentTasks.length === 0" class="recent-empty">暂无任务</div>
        </div>
      </div>
    </div>

    <!-- 最近活动 -->
    <div class="activity-section">
      <div class="activity-title">最近活动</div>
      <div class="activity-list">
        <div v-for="t in recentTasks" :key="t.id" class="activity-row" @click="openTask(t.id)">
          <span class="activity-badge" :style="{ background: typeColors[t.task_type] + '20', color: typeColors[t.task_type] }">{{ TASK_TYPE_ICONS[t.task_type] }}</span>
          <div class="activity-info">
            <span class="activity-name">{{ t.name }}</span>
            <span class="activity-meta">{{ TASK_TYPE_LABELS[t.task_type] }} · {{ t.stats.annotated_images }}/{{ t.stats.total_images }} 已标注</span>
          </div>
          <span class="activity-time">{{ formatTime(t.updated_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from "vue";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { BarChart, PieChart } from "echarts/charts";
import { GridComponent, TooltipComponent, LegendComponent } from "echarts/components";
import VChart from "vue-echarts";
import { useProjectStore } from "@/stores/project";
import { TASK_TYPE_LABELS, TASK_TYPE_ICONS, type TaskType } from "@/utils/taskTypes";

use([CanvasRenderer, BarChart, PieChart, GridComponent, TooltipComponent, LegendComponent]);

const projectStore = useProjectStore();

const typeColors: Record<string, string> = {
  classification: "#6366f1", detection: "#3b82f6", rotated_detection: "#8b5cf6",
  keypoint: "#eab308", segmentation: "#22c55e", ocr: "#06b6d4",
};

const stats = computed(() => {
  const tasks = projectStore.tasks;
  const totalImgs = tasks.reduce((s, t) => s + (t.stats?.total_images ?? 0), 0);
  const totalAnn = tasks.reduce((s, t) => s + (t.stats?.annotated_images ?? 0), 0);
  const allClasses = new Set(tasks.flatMap(t => (t.classes ?? []).map(c => c.name)));
  return [
    { icon: "📋", label: "总任务", value: tasks.length, changeText: `${tasks.length}`, color: "#6366f1", change: 0 },
    { icon: "🖼️", label: "总图片", value: totalImgs, changeText: `${totalImgs}`, color: "#3b82f6", change: 0 },
    { icon: "🏷️", label: "总类别", value: allClasses.size, changeText: `${allClasses.size}`, color: "#22c55e", change: 0 },
    { icon: "✅", label: "已标注", value: totalAnn, changeText: `${totalAnn}`, color: "#f97316", change: 0 },
  ];
});

// 各类别标注数量柱状图
const barOption = computed(() => ({
  tooltip: { trigger: "axis" as const },
  grid: { left: 40, right: 16, top: 8, bottom: 28 },
  xAxis: { type: "category" as const, data: typeNames(), axisLabel: { fontSize: 10, color: "#a1a1aa" } },
  yAxis: { type: "value" as const, splitLine: { lineStyle: { color: "#3e3e42" } }, axisLabel: { fontSize: 10 } },
  series: [{ type: "bar" as const, data: typeAnns(), itemStyle: { color: (p: any) => typeColors[Object.keys(typeColors)[p.dataIndex]] || "#a1a1aa", borderRadius: [4,4,0,0] }, barWidth: "60%" }],
}));

// 标注进度环形图
const donutOption = computed(() => {
  const total = stats.value[1].value - stats.value[3].value;
  return {
    tooltip: { trigger: "item" as const },
    series: [{
      type: "pie" as const, radius: ["55%", "75%"], avoidLabelOverlap: false,
      center: ["50%", "50%"],
      label: { show: true, position: "center", formatter: () => `${stats.value[3].value}/${stats.value[1].value}`, fontSize: 16, fontWeight: 700, color: "#e4e4e7" },
      emphasis: { label: { show: true, fontSize: 18 } },
      data: [
        { value: stats.value[3].value, name: "已标注", itemStyle: { color: "#f97316" } },
        { value: Math.max(0, total), name: "未标注", itemStyle: { color: "#3e3e42" } },
      ],
    }],
  };
});

// 各类别图片分布柱状图
const imgBarOption = computed(() => ({
  tooltip: { trigger: "axis" as const },
  grid: { left: 40, right: 16, top: 8, bottom: 28 },
  xAxis: { type: "category" as const, data: typeNames(), axisLabel: { fontSize: 10, color: "#a1a1aa" } },
  yAxis: { type: "value" as const, splitLine: { lineStyle: { color: "#3e3e42" } }, axisLabel: { fontSize: 10 } },
  series: [{ type: "bar" as const, data: typeImages(), itemStyle: { color: (p: any) => typeColors[Object.keys(typeColors)[p.dataIndex]] || "#a1a1aa", borderRadius: [4,4,0,0] }, barWidth: "60%" }],
}));

const recentTasks = computed(() =>
  [...projectStore.tasks].sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()).slice(0, 6)
);

function typeNames() {
  return Object.keys(typeColors).map(k => TASK_TYPE_LABELS[k as TaskType] || k);
}

function typeAnns(): number[] {
  const m: Record<string, number> = {};
  projectStore.tasks.forEach(t => { m[t.task_type] = (m[t.task_type] || 0) + (t.stats?.annotated_images ?? 0); });
  return Object.keys(typeColors).map(k => m[k] || 0);
}

function typeImages(): number[] {
  const m: Record<string, number> = {};
  projectStore.tasks.forEach(t => { m[t.task_type] = (m[t.task_type] || 0) + (t.stats?.total_images ?? 0); });
  return Object.keys(typeColors).map(k => m[k] || 0);
}

function openTask(id: string) {
  projectStore.openTask(id);
}

function formatTime(iso: string): string {
  if (!iso) return "-";
  try { return new Date(iso).toLocaleDateString("zh-CN"); } catch { return iso; }
}
</script>

<style scoped>
.dashboard {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 统计卡片 */
.stat-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}
.stat-card {
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  padding: 18px 20px;
  display: flex;
  align-items: center;
  gap: 14px;
}
.stat-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  flex-shrink: 0;
}
.stat-body { flex: 1; }
.stat-value { font-size: 24px; font-weight: 700; color: var(--text-primary); line-height: 1.2; }
.stat-label { font-size: 12px; color: var(--text-secondary); margin-top: 2px; }
.stat-change { font-size: 12px; font-weight: 600; }

/* 图表 */
.chart-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}
.chart-card {
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  padding: 16px;
}
.chart-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

/* 最近编辑列表 */
.recent-list { display: flex; flex-direction: column; gap: 6px; }
.recent-item {
  display: flex; align-items: center; gap: 10px; padding: 8px 10px;
  border-radius: 6px; cursor: pointer; transition: background 0.12s;
}
.recent-item:hover { background: var(--bg-hover); }
.recent-type { font-size: 16px; width: 24px; text-align: center; }
.recent-name { flex: 1; font-size: 12px; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.recent-time { font-size: 11px; color: var(--text-dim); flex-shrink: 0; }
.recent-empty { padding: 20px; text-align: center; color: var(--text-dim); font-size: 12px; }

/* 活动列表 */
.activity-section { background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: 10px; padding: 16px; }
.activity-title { font-size: 13px; font-weight: 600; color: var(--text-primary); margin-bottom: 8px; }
.activity-list { display: flex; flex-direction: column; gap: 2px; }
.activity-row {
  display: flex; align-items: center; gap: 12px; padding: 10px 12px;
  border-radius: 6px; cursor: pointer; transition: background 0.12s;
}
.activity-row:hover { background: var(--bg-hover); }
.activity-badge { width: 32px; height: 32px; border-radius: 6px; display: flex; align-items: center; justify-content: center; font-size: 15px; flex-shrink: 0; }
.activity-info { flex: 1; }
.activity-name { font-size: 13px; font-weight: 500; color: var(--text-primary); display: block; }
.activity-meta { font-size: 11px; color: var(--text-dim); }
.activity-time { font-size: 11px; color: var(--text-dim); flex-shrink: 0; }
</style>