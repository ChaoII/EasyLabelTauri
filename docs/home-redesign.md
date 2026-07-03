# EasyLabel 首页/工作台重设计

## 整体布局

```
+----------+--------------------------------------------------+
| NavSidebar | AppHeader (EasyLabel  |  中心插槽  |  主题切换)   |
| (62px)     +--------------------------------------------------+
| [E logo]  |                                                  |
|           |  主内容区域                                        |
| [首页]    |  • 首页：统计仪表盘（卡片 + 图表）                  |
| [标注]    |  • 标注工作台：任务卡片列表（搜索/筛选/分页）       |
|           |  • 任务标注：AnnotationView（画布+工具）            |
|           |                                                  |
| [⚙设置]   |                                                  |
+----------+--------------------------------------------------+
```

## 导航栏 (NavSidebar)
- 顶部：E logo 图标（EL 品牌标识，橙色背景）
- 中部：两个导航按钮（首页 / 标注工作台），带 active 指示条
- 底部：设置齿轮图标（打开设置弹窗）

## 顶部栏 (AppHeader)
- 左侧：EasyLabel 名称（不再显示 logo，因为已在侧栏）
- 中间：插槽（各页面自定义标题）
- 右侧：主题切换按钮（替代原来的设置按钮）

## 首页 — 统计仪表盘

### 顶部统计卡片（4 张数字卡）
| 指标 | 来源 |
|------|------|
| 📋 总任务数 | projectStore.tasks.length |
| 🖼️ 总图片数 | tasks.map(t => t.stats.total_images).reduce |
| 🏷️ 总类别数 | tasks.flatMap(t => t.classes).length (去重) |
| ✅ 已标注数 | tasks.map(t => t.stats.annotated_images).reduce |

### 图表区（4 个图表）
使用轻量级 Canvas 图表（手绘 SVG 或 tiny-chart），无需额外依赖：
1. **各类别标注数量柱状图** — 各 task_type 的 total_annotations 汇总
2. **标注进度环形图** — 已标注图片数 / 总图片数
3. **各类别图片数柱状图** — 各 task_type 的 total_images 汇总
4. **最近编辑任务列表** — 按 updated_at 排序，前 5 条

### 布局
- 统计卡片：2x2 网格
- 图表：2x2 网格（每个图表约 240px 高）
- 底部：最近任务列表

## 标注工作台 — 任务卡片列表
- 原有的 HomeView 内容（搜索 / 类型筛选 / 排序 / 卡片 / 分页）
- 顶部增加「最近标注」快速跳转行（显示最近 5 个任务）

## 实现计划
1. 创建 `HomeDashboard.vue` — 统计仪表盘
2. 创建 `WorkspaceView.vue` — 任务卡片列表（从 HomeView 抽取）
3. 修改 `HomeView.vue` — 根据 currentView 切换仪表盘或工作台
4. 更新 `App.vue` — 集成 currentView 状态
5. 测试验证
