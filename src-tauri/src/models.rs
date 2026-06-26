use serde::{Deserialize, Serialize};

/// 关键点可见性
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
    Hidden = 0,
    Occluded = 1,
    Visible = 2,
}

/// 2D 点（归一化坐标 0-1）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// 普通矩形框（轴对齐）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisAlignedBox {
    pub id: String,
    pub class_id: usize,
    pub x1: f64,  // 左上角 x (归一化 0-1)
    pub y1: f64,  // 左上角 y (归一化 0-1)
    pub x2: f64,  // 右下角 x (归一化 0-1)
    pub y2: f64,  // 右下角 y (归一化 0-1)
    pub confidence: f64,
}

/// 旋转矩形 (YOLO 旋转框格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotatedBox {
    pub id: String,
    pub class_id: usize,
    pub cx: f64,     // 中心 x (归一化 0-1)
    pub cy: f64,     // 中心 y (归一化 0-1)
    pub width: f64,  // 宽度 (归一化 0-1)
    pub height: f64, // 高度 (归一化 0-1)
    pub angle: f64,  // 角度 (弧度)
    pub confidence: f64,
}

/// 多边形分割
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonAnnotation {
    pub id: String,
    pub class_id: usize,
    pub points: Vec<Point>,
    pub holes: Vec<Vec<Point>>,
    pub confidence: f64,
}

/// 关键点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keypoint {
    pub x: f64,
    pub y: f64,
    pub visibility: Visibility,
    pub name: String,
}

/// 关键点标注
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeypointAnnotation {
    pub id: String,
    pub class_id: usize,
    pub keypoints: Vec<Keypoint>,
    pub bounding_box: Option<RotatedBox>,
    pub confidence: f64,
}

/// OCR 标注
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrAnnotation {
    pub id: String,
    pub class_id: usize,
    pub points: Vec<Point>,
    pub text: String,
    pub confidence: f64,
}

/// 分类标注
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationAnnotation {
    pub id: String,
    pub class_ids: Vec<usize>,
}

/// 统一标注类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Annotation {
    AxisAlignedBox(AxisAlignedBox),
    RotatedBox(RotatedBox),
    Polygon(PolygonAnnotation),
    Keypoint(KeypointAnnotation),
    Classification(ClassificationAnnotation),
    Ocr(OcrAnnotation),
}

impl Annotation {
    pub fn get_id(&self) -> &str {
        match self {
            Annotation::AxisAlignedBox(b) => &b.id,
            Annotation::RotatedBox(rb) => &rb.id,
            Annotation::Polygon(p) => &p.id,
            Annotation::Keypoint(k) => &k.id,
            Annotation::Classification(c) => &c.id,
            Annotation::Ocr(o) => &o.id,
        }
    }

    pub fn get_class_id(&self) -> usize {
        match self {
            Annotation::AxisAlignedBox(b) => b.class_id,
            Annotation::RotatedBox(rb) => rb.class_id,
            Annotation::Polygon(p) => p.class_id,
            Annotation::Keypoint(k) => k.class_id,
            Annotation::Classification(c) => c.class_ids.first().copied().unwrap_or(0),
            Annotation::Ocr(o) => o.class_id,
        }
    }
}

/// 类别定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDefinition {
    pub id: usize,
    pub name: String,
    pub color: String,
    pub keypoint_names: Option<Vec<String>>,
    pub keypoint_count: Option<usize>,
    pub keypoint_colors: Option<Vec<String>>,
}

impl ClassDefinition {
    pub fn new(id: usize, name: String, color: String) -> Self {
        Self { id, name, color, keypoint_names: None, keypoint_count: None, keypoint_colors: None }
    }
}

/// 项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub image_path: String,
    pub image_width: u32,
    pub image_height: u32,
    pub annotations: Vec<Annotation>,
    pub classes: Vec<ClassDefinition>,
    pub classification_labels: Vec<usize>,
}

impl Project {
    pub fn new(image_path: String, width: u32, height: u32) -> Self {
        Self {
            name: "Unnamed Project".to_string(),
            image_path,
            image_width: width,
            image_height: height,
            annotations: Vec::new(),
            classes: Self::default_classes(),
            classification_labels: Vec::new(),
        }
    }

    pub fn default_classes() -> Vec<ClassDefinition> {
        vec![
            ClassDefinition::new(0, "行人".to_string(), "#38bdf8".to_string()),
            ClassDefinition::new(1, "车辆".to_string(), "#3b82f6".to_string()),
            ClassDefinition::new(2, "文字".to_string(), "#eab308".to_string()),
            ClassDefinition::new(3, "物体".to_string(), "#4ade80".to_string()),
            ClassDefinition::new(4, "人脸".to_string(), "#f472b6".to_string()),
        ]
    }

    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    pub fn remove_annotation(&mut self, id: &str) {
        self.annotations.retain(|a| a.get_id() != id);
    }
}

/// 工具类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToolType {
    Select,
    Pan,
    RotatedBox,
    Polygon,
    Keypoint,
    Ocr,
    Classification,
}

impl ToolType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "select" => ToolType::Select,
            "pan" => ToolType::Pan,
            "rotated_box" => ToolType::RotatedBox,
            "polygon" => ToolType::Polygon,
            "keypoint" => ToolType::Keypoint,
            "ocr" => ToolType::Ocr,
            "classification" => ToolType::Classification,
            _ => ToolType::Select,
        }
    }
}

/// 旋转矩形工具状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotatedBoxToolState {
    pub is_drawing: bool,
    pub start_x: f64,
    pub start_y: f64,
    pub current_x: f64,
    pub current_y: f64,
    pub angle: f64,
}

impl Default for RotatedBoxToolState {
    fn default() -> Self {
        Self { is_drawing: false, start_x: 0.0, start_y: 0.0, current_x: 0.0, current_y: 0.0, angle: 0.0 }
    }
}

/// 多边形工具状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonToolState {
    pub points: Vec<Point>,
    pub is_finished: bool,
}

impl Default for PolygonToolState {
    fn default() -> Self {
        Self { points: Vec::new(), is_finished: false }
    }
}

/// 应用状态（持久化部分）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateData {
    pub current_project: Option<Project>,
    pub active_tool: ToolType,
    pub selected_annotation_id: Option<String>,
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub image_base64: Option<String>,
    pub classes: Vec<ClassDefinition>,
    pub rotated_box_tool: RotatedBoxToolState,
    pub polygon_tool: PolygonToolState,
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 主题模式: dark / light / system
    pub theme_mode: String,
    /// 主题色
    pub accent_color: String,
    /// 标注线宽: thin / medium / thick
    pub annotation_line_width: String,
    /// 是否显示标签
    pub show_labels: bool,
    /// 是否显示状态栏
    pub status_bar_visible: bool,
    /// 紧凑模式
    pub dense_mode: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: "dark".to_string(),
            accent_color: "#f97316".to_string(),
            annotation_line_width: "medium".to_string(),
            show_labels: true,
            status_bar_visible: true,
            dense_mode: false,
        }
    }
}

impl Default for AppStateData {
    fn default() -> Self {
        Self {
            current_project: None,
            active_tool: ToolType::Select,
            selected_annotation_id: None,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            image_base64: None,
            classes: Project::default_classes(),
            rotated_box_tool: RotatedBoxToolState::default(),
            polygon_tool: PolygonToolState::default(),
        }
    }
}
