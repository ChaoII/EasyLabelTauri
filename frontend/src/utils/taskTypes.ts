import type { ClassDefinition } from "./types";

// ==================== 任务类型 ====================

/**
 * 任务类型枚举
 */
export type TaskType =
  | "classification"   // 多标签分类
  | "detection"        // 目标检测（普通矩形框）
  | "rotated_detection" // 旋转框检测
  | "keypoint"          // 关键点检测（角点 + 矩形框）
  | "segmentation"      // 实例分割（多边形）
  | "ocr";              // OCR 文本识别

/**
 * 任务类型中文名
 */
export const TASK_TYPE_LABELS: Record<TaskType, string> = {
  classification: "多标签分类",
  detection: "目标检测",
  rotated_detection: "旋转框检测",
  keypoint: "关键点检测",
  segmentation: "实例分割",
  ocr: "OCR 文本识别",
};

/**
 * 任务类型图标
 */
export const TASK_TYPE_ICONS: Record<TaskType, string> = {
  classification: "☰",
  detection: "□",
  rotated_detection: "◇",
  keypoint: "✦",
  segmentation: "⬡",
  ocr: "T",
};

// ==================== 任务 ====================

export interface Task {
  id: string;
  name: string;
  task_type: TaskType;
  created_at: string;   // ISO 时间字符串
  updated_at: string;   // ISO 时间字符串
  image_folder: string; // 图片目录路径
  classes: ClassDefinition[]; // 类别定义
  stats: TaskStats;      // 标注统计
}

export interface TaskStats {
  total_images: number;       // 总图片数
  annotated_images: number;   // 已标注图片数
  total_annotations: number; // 总标注数
}

// ==================== 项目配置 ====================

export interface AnnotationConfig {
  // 该任务启用的标注类型
  annotation_types: TaskType[];
}

// ==================== 任务列表 ====================

export interface TaskListData {
  tasks: Task[];
}
