# EasyLabelSlint - 数据标注工具规格说明书

## 1. 项目概述

**项目名称:** EasyLabelSlint
**项目类型:** 桌面端数据标注工具
**核心功能:** 支持 YOLO 格式的旋转矩形、分割、关键点、分类和 OCR 标注的多功能标注工具
**目标用户:** 计算机视觉工程师、数据标注员、AI 研究人员

## 2. 技术栈

- **前端框架:** Vue 3 + TypeScript + Vite + Pinia
- **后端框架:** Tauri 2.x (Rust)
- **编程语言:** Rust + TypeScript
- **构建工具:** Cargo + npm
- **图形渲染:** HTML5 Canvas + SVG (前端)
- **系统集成:** Tauri 插件 (文件对话框、文件系统)

### 目录结构

```
EasyLabelSlint/
├── frontend/                   # Vue 3 前端
│   ├── src/
│   │   ├── App.vue           # 主应用组件
│   │   ├── main.ts          # 入口文件
│   │   ├── assets/
│   │   │   └── main.css    # 全局样式
│   │   ├── components/
│   │   │   ├── Canvas.vue   # 画布（图片+标注渲染）
│   │   │   ├── ToolButton.vue
│   │   │   ├── ClassList.vue
│   │   │   └── AnnotationList.vue
│   │   ├── stores/
│   │   │   └── app.ts      # Pinia 状态管理
│   │   └── utils/
│   │       └── types.ts    # TypeScript 类型定义
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
├── src-tauri/                 # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs          # 程序入口
│   │   ├── lib.rs          # 库入口
│   │   ├── models.rs       # 数据模型（标注/项目）
│   │   └── commands.rs     # Tauri 命令（与前端通信）
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   ├── build.rs
│   └── icons/
├── Cargo.toml                # 旧 Rust 项目（已弃用）
├── src/                      # 旧源码（已弃用）
├── ui/                       # 旧 Slint UI（已弃用）
└── SPEC.md
```

## 3. 标注类型支持

### 3.1 旋转矩形 (Rotated Box)
- 支持任意角度旋转的边界框
- 格式: (cx, cy, w, h, angle) 归一化至 [0,1]
- 可拖拽调整 4 个控制点
- 可旋转手柄调整角度

### 3.2 分割标注 (Segmentation)
- 多边形顶点序列
- 格式: [x1,y1,x2,y2,...,xn,yn] 归一化
- 支持添加/删除顶点
- 支持闭合多边形

### 3.3 关键点标注 (Keypoints)
- 每个目标支持 N 个关键点
- 格式: [x1,y1,v1,x2,y2,v2,...] (v=可见性 0/1/2)
- 可拖拽关键点位置
- 支持设置可见性状态

### 3.4 分类标注 (Classification)
- 图像级标签
- 支持多标签分类
- 下拉选择或复选框形式

### 3.5 OCR 标注 (OCR)
- 矩形区域文本标注
- 支持弯曲文本 (四点定位)
- 文本内容输入框

## 4. UI/UX 设计规范

### 4.1 整体视觉风格
- **主题:** 深色专业工具风格（类 VS Code / VisionLabel）
- **主色调:** 中性深灰 (#1e1e1e / #252526 / #2d2d30)
- **强调色:** 橙色 (#f97316) 用于选中和主要操作
- **边框色:** (#3e3e42)
- **字体:** 系统默认无衬线字体

### 4.2 配色方案
```
背景主色:     #1e1e1e (App 背景)
面板色:       #252526 (顶栏/侧栏)
高亮色:       #2d2d30 (卡片/按钮)
边框色:       #3e3e42 (分隔线)
强调色:       #f97316 (橙色选中)
文本主色:     #e4e4e7 (白色文字)
文本次色:     #a1a1aa (灰色说明)
文字暗淡:     #71717a (辅助信息)
成功色:       #4ade80 (绿色)
危险色:       #f87171 (删除/错误)
```

### 4.3 布局结构
```
+--------------------------------------------------+
|  顶部工具栏 (Logo + 打开/撤销/导出 + 状态 + 缩放) |
+--------+----------------------------------+-------+
|        |                                  |       |
| 左侧  |        主画布区域               | 右侧  |
| 工具  |        (图片 + SVG 标注叠加)     | 面板  |
| 52px  |        支持鼠标绘制交互         | 300px |
|        |                                  | 图片  |
|  7个  |                                  | 类别  |
| 图标  |                                  | 标注  |
|        |                                  | 列表  |
+--------+----------------------------------+-------+
|  底部状态栏 (提示信息 + 坐标 + 保存)           |
+--------------------------------------------------+
```

## 5. 功能模块

### 5.1 文件操作
- 打开图像/图像文件夹（系统文件选择对话框）
- 导出 YOLO 格式 (.txt)
- 导入已有标注
- 保存/加载 JSON 项目文件

### 5.2 标注操作
- 创建新标注（旋转矩形、多边形等）
- 编辑现有标注
- 删除标注
- 复制/粘贴标注
- 撤销/重做（占位）

### 5.3 导航操作
- 平移画布
- 缩放 (滚轮)
- 适应窗口
- 100% 显示

### 5.4 标注管理
- 标注列表显示（可点击选择/删除）
- 类别管理（默认 5 类：行人、车辆、文字、物体、人脸）
- 标注过滤显示

## 6. 导出格式

### YOLO Detection (普通)
```
<class_id> <x_center> <y_center> <width> <height>
```

### YOLO Detection (旋转)
```
<class_id> <cx> <cy> <w> <h> <angle>
```

### YOLO Segmentation
```
<class_id> <x1> <y1> <x2> <y2> ... <xn> <yn>
```

### YOLO Keypoints
```
<class_id> <cx> <cy> <w> <h> <kx1> <ky1> <v1> ...
```

### Classification
```
<class_name1>
<class_name2>
...
```

## 7. 快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+O | 打开图像 |
| Ctrl+S | 保存标注 |
| Ctrl+Z | 撤销 |
| Ctrl+Y | 重做 |
| V | 选择工具 |
| R | 旋转矩形工具 |
| P | 多边形工具 |
| K | 关键点工具 |
| T | 文本/OCR 工具 |
| C | 分类工具 |
| Delete | 删除选中标注 |
| +/- | 缩放 |
| 空格+拖拽 | 平移 |

## 8. Tauri 命令接口

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `load_image` | `path: String` | `LoadImageResult` | 加载图片并返回 base64 |
| `save_project` | `path: String, project: Project` | `()` | 保存项目到 JSON |
| `load_project` | `path: String` | `Project` | 从 JSON 加载项目 |
| `export_yolo` | `output_dir: String, project: Project` | `()` | 导出 YOLO 格式 |
| `create_rotated_box` | `class_id, x1, y1, x2, y2, angle` | `Annotation` | 创建旋转框 |
| `create_polygon` | `class_id, points: Vec<Point>` | `Option<Annotation>` | 创建多边形 |
| `create_classification` | `class_ids: Vec<usize>` | `Annotation` | 创建分类标注 |
| `get_default_classes` | - | `Vec<ClassDefinition>` | 获取默认类别列表 |

## 9. 验收标准

- [x] 支持打开图像文件 (PNG, JPG, BMP, WebP, TIFF)
- [x] 支持旋转矩形标注（在画布上拖拽绘制）
- [x] 支持多边形分割标注（点击添加顶点，双击闭合）
- [ ] 支持关键点标注（数据结构已实现，UI 交互待完善）
- [ ] 支持分类标注（数据结构已实现，UI 交互待完善）
- [ ] 支持 OCR 文本标注（数据结构已实现，UI 交互待完善）
- [x] 支持 YOLO 格式导出
- [x] 应用程序可正常编译运行
- [ ] UI 呈现专业深色工具风格（已实现基础布局）
- [ ] 撤销/重做功能
- [ ] 拖放图片到窗口
