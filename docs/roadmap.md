# EasyLabel 工业级标注平台 — 完整路线图

## 概述
将 EasyLabel 从当前的功能性标注工具升级为工业级数据标注平台。分 6 个阶段推进，每个阶段独立可交付。

---

## P1：核心标注增强（优先级最高）

### 1.1 撤销/重做
- **范围**：仅标注操作（添加 `addAnnotation`、删除 `removeAnnotation`、修改 `updateAnnotation`）
- **实现**：在 `app.ts` 中维护 `history: Annotation[][]` 和 `historyIndex: number`
  - 每次标注变更前，将当前快照推入历史栈
  - `Ctrl+Z`：`historyIndex--`，恢复历史快照
  - `Ctrl+Shift+Z` / `Ctrl+Y`：`historyIndex++`，恢复快照
  - 最大 50 步
- **UI**：左侧工具栏底部增加撤销/重做图标按钮
- **快捷键**：已在 AnnotationView 的 `onKeyDown` 中处理

### 1.2 复制粘贴
- **范围**：当前图片内复制
- **快捷键**：`Ctrl+C` 复制选中标注 → `Ctrl+V` 粘贴（偏移 15px 避免重叠）
- **实现**：`copySelectedAnnotation()` → 深拷贝，生成新 UUID，偏移坐标
- **UI**：标注列表右键菜单（可选）

### 1.3 批量操作
- **多选**：标注列表支持 `Ctrl+点击` 多选，选中项高亮
- **批量删除**：选中多个 → 右键 → 删除 / `Delete` 键批量删除
- **批量改类**：选中多个 → 右键 → 修改类别 → 弹出 NSelect 选择器

### 1.4 标注锁定
- **数据模型**：`Annotation` 增加 `locked: boolean` 字段（Rust 和 TypeScript 同步）
- **UI**：标注列表右侧加锁图标，点击切换
- **行为**：锁定标注不可选中、不可拖拽、不可编辑、不可删除

### 1.5 标注搜索
- **搜索 OCR 文本**：标注列表顶部加搜索框，过滤 OCR 文本内容
- **按类别筛选**：NSelect 下拉选择类别

---

## P2：数据导入

### 2.1 YOLO 导入
- 读取 `.txt` 文件，解析 `class_id x_center y_center w h`
- 转换为 `AxisAlignedBox` 标注
- 支持 detection / rotated_detection / segmentation 格式

### 2.2 COCO JSON 导入
- 读取 `annotations.json`，解析 `images`、`annotations`、`categories`
- 转换为 `AxisAlignedBox`、`PolygonAnnotation`、`KeypointAnnotation`

### 2.3 PaddleOCR 导入
- 读取 `train.txt` / `val.txt`，解析 `image_path\t[{"transcription":..., "points":[...]}]`
- 转换为 `OcrAnnotation`

### 2.4 UI
- 任务卡片（工作台视图）增加「导入」按钮
- 弹窗选择格式 + 文件路径
- 进度条显示导入进度

---

## P3：数据管理

### 3.1 数据集分割
- 在导出时增加 train/val/test 比例配置
- 支持按文件列表随机分割

### 3.2 数据验证
- 检查标注是否有越界坐标（超出 [0,1]）
- 检查重叠框
- 检查空标注图片
- 报告汇总

### 3.3 导出历史
- 在 `projects.json` 中记录导出操作（时间、格式、路径、数量）
- 首页显示最近的导出记录

---

## P4：模型管理

### 4.1 模型配置界面
- 侧栏新增「模型」导航
- 模型列表：名称、类型、路径、状态
- 添加/删除模型配置

### 4.2 模型评估
- 在标注数据上运行模型推理
- 对比自动标注 vs 人工标注
- 输出 mAP、Precision、Recall 等指标

### 4.3 自动标注集成
- 在标注界面的 AI 标注弹窗中，直接选择已配置的模型
- 不再需要每次手动选择 ONNX 文件

---

## P5：用户体验

### 5.1 多语言 (i18n)
- 使用 Vue I18n 或自建轻量方案
- 支持中文/英文切换
- 当前所有中文文本抽离为 i18n key

### 5.2 性能优化
- 虚拟列表：图片列表和标注列表使用虚拟滚动
- 延迟加载：按需加载标注文件，不一次性加载全部
- 图片缓存：使用 LRU 缓存已加载的图片

---

## P6：测试体系

### 6.1 单元测试
- Pinia store 测试（Vitest）
- 工具函数测试
- Rust 函数测试

### 6.2 集成测试
- 标注创建/修改/删除流程
- 导出流程
- AI 自动标注流程

### 6.3 E2E 测试
- Playwright 或 WebDriver 测试
- 关键用户路径测试

---

## 实施顺序

```
P1 → P2 → P3 → P4 → P5 → P6
```

每阶段完成后交付、测试、确认再进入下一阶段。