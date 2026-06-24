# EasyLabelSlint

> 科幻风格的数据标注工具 | Cyberpunk Data Labeling Tool

## 功能特性

- **旋转矩形标注** - 支持任意角度旋转的边界框 (Rotated Box)
- **多边形分割标注** - 自由绘制多边形区域进行分割标注
- **关键点标注** - 支持17点人体关键点及自定义关键点
- **OCR文本标注** - 四点定位弯曲文本区域标注
- **图像分类** - 支持多标签图像分类
- **YOLO格式导出** - 支持YOLO标准格式的导入导出

## 界面预览

界面采用赛博朋克/科幻风格设计，包含：
- 深空黑主色调 + 霓虹蓝/紫/绿配色
- 霓虹发光边框效果
- 科技感字体和图标
- 流畅的交互动画

## 技术栈

- **UI框架**: Slint 1.8+ (Rust)
- **后端语言**: Rust
- **图像处理**: image crate
- **数据格式**: JSON + YOLO TXT

## 项目结构

```
EasyLabelSlint/
├── Cargo.toml           # Rust 项目配置
├── SPEC.md              # 项目规格说明
├── README.md            # 项目文档
├── src/
│   ├── main.rs          # 程序入口
│   ├── lib.rs           # 库入口
│   ├── app.rs           # 应用状态管理
│   ├── models/
│   │   └── mod.rs       # 数据模型定义
│   ├── tools/
│   │   └── mod.rs       # 标注工具实现
│   └── io/
│       └── mod.rs       # YOLO格式导入导出
└── ui/
    └── main.slint        # Slint UI 定义
```

## 编译运行

### 环境要求

- Rust 1.70+
- Cargo 包管理器
- Windows/Linux/macOS

### 编译步骤

```bash
# 克隆项目
git clone https://github.com/your-repo/EasyLabelSlint.git
cd EasyLabelSlint

# 编译项目
cargo build --release

# 运行程序
cargo run --release
```

### 注意事项

- 首次编译需要下载依赖，请保持网络连接
- 编译过程可能需要几分钟时间
- 如果遇到权限问题，请以管理员身份运行

## 使用说明

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `V` | 选择工具 |
| `R` | 旋转矩形工具 |
| `P` | 多边形分割工具 |
| `K` | 关键点标注工具 |
| `T` | OCR 文本标注 |
| `C` | 分类标注 |
| `Delete` | 删除选中标注 |
| `+/-` | 缩放 |
| `Space` + 拖拽 | 平移画布 |

### 标注流程

1. **打开图像**: 点击"打开图像"按钮选择图片
2. **选择工具**: 在左侧工具栏选择标注类型
3. **绘制标注**: 在画布上绘制标注区域
4. **设置类别**: 在右侧面板设置标注类别
5. **保存/导出**: 导出为YOLO格式

### YOLO 格式说明

#### 旋转矩形格式
```
<class_id> <cx> <cy> <width> <height> <angle>
```
- 所有坐标值均为归一化值 (0-1)
- angle 为弧度制

#### 分割格式
```
<class_id> <x1> <y1> <x2> <y2> ... <xn> <yn>
```

#### 关键点格式
```
<class_id> <cx> <cy> <w> <h> <kx1> <ky1> <v1> ...
```
- v 为可见性: 0=不可见, 1=遮挡, 2=可见

#### 分类格式
```
<class_name>
```

## 开发指南

### 添加新的标注类型

1. 在 `src/models/mod.rs` 中定义新的数据结构
2. 在 `src/tools/mod.rs` 中实现标注工具
3. 在 `src/io/mod.rs` 中实现格式导出
4. 在 `ui/main.slint` 中添加UI组件

### 自定义配色

修改 `ui/main.slint` 中的 `Theme` 全局样式：
- `bg-primary`: 主背景色
- `accent-cyan`: 霓虹蓝
- `accent-purple`: 霓虹紫
- `accent-green`: 霓虹绿

## 许可证

MIT License

## 联系方式

- GitHub Issues: [提交问题](https://github.com/your-repo/EasyLabelSlint/issues)
