// ==================== 标注类型 ====================

export interface Point {
  x: number;
  y: number;
}

export type Visibility = "Hidden" | "Occluded" | "Visible";

export interface Keypoint {
  x: number;
  y: number;
  visibility: Visibility;
  name: string;
}

export interface AxisAlignedBox {
  id: string;
  type: "AxisAlignedBox";
  class_id: number;
  x1: number; // 左上角 x (归一化 0-1)
  y1: number; // 左上角 y (归一化 0-1)
  x2: number; // 右下角 x (归一化 0-1)
  y2: number; // 右下角 y (归一化 0-1)
  confidence: number;
}

export interface RotatedBox {
  id: string;
  type: "RotatedBox";
  class_id: number;
  cx: number;
  cy: number;
  width: number;
  height: number;
  angle: number;
  confidence: number;
}

export interface PolygonAnnotation {
  id: string;
  type: "Polygon";
  class_id: number;
  points: Point[];
  holes: Point[][];
  confidence: number;
}

export interface KeypointAnnotation {
  id: string;
  type: "Keypoint";
  class_id: number;
  keypoints: Keypoint[];
  bounding_box: RotatedBox | null;
  confidence: number;
}

export interface OcrAnnotation {
  id: string;
  type: "Ocr";
  class_id: number;
  points: Point[];
  text: string;
  confidence: number;
}

export interface ClassificationAnnotation {
  id: string;
  type: "Classification";
  class_ids: number[];
}

export type Annotation =
  | AxisAlignedBox
  | RotatedBox
  | PolygonAnnotation
  | KeypointAnnotation
  | OcrAnnotation
  | ClassificationAnnotation;

export interface ClassDefinition {
  id: number;
  name: string;
  color: string;
  keypoint_names?: string[];
  keypoint_count?: number;
  keypoint_colors?: string[];
}

// ==================== 工具 ====================

export type ToolName =
  | "select"
  | "box"
  | "rotated_box"
  | "polygon"
  | "keypoint"
  | "ocr"
  | "pan"
  | "zoom"
  | "classification";

/** 底栏、提示等用的短中文名 */
export const TOOL_LABELS: Record<ToolName, string> = {
  select: "选择",
  box: "矩形",
  rotated_box: "旋转框",
  polygon: "多边形",
  keypoint: "关键点",
  ocr: "OCR",
  pan: "平移",
  zoom: "缩放",
  classification: "分类",
};

// ==================== 状态 ====================

export interface Project {
  name: string;
  image_path: string;
  image_width: number;
  image_height: number;
  annotations: Annotation[];
  classes: ClassDefinition[];
  classification_labels: number[];
}

// ==================== Rust 返回 ====================

export interface LoadImageResult {
  /** 预留；当前为空，画布使用 image_path + convertFileSrc 加载原图 */
  base64: string;
  width: number;
  height: number;
  image_path: string;
}

// ==================== 文件夹/图片列表 ====================

export interface ImageInfo {
  index: number;
  name: string;
  path: string;
}

export interface FolderInfo {
  folder_path: string;
  images: ImageInfo[];
}
