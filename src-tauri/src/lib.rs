//! EasyLabel — 数据标注工具核心库
//! 提供图像处理、标注管理、YOLO 导入导出、AI 自动标注

pub mod models;
pub mod commands;
pub mod auto_annotate;

pub use commands::{FolderInfo, ImageInfo, load_folder, load_annotations_for_image, save_annotations_for_image, load_settings, save_settings};
pub use models::*;
