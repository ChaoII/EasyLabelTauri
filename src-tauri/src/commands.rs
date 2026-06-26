//! Tauri Commands — 与前端通信的接口
use crate::models::*;
use serde_json;
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

// ==================== 文件夹/图片列表 ====================

const SUPPORTED_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "bmp", "gif", "webp", "tiff", "tif"];

fn is_image_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[derive(serde::Serialize)]
pub struct FolderInfo {
    pub folder_path: String,
    pub images: Vec<ImageInfo>,
}

#[derive(serde::Serialize)]
pub struct ImageInfo {
    pub index: usize,
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub fn load_folder(folder_path: String) -> Result<FolderInfo, String> {
    let path = Path::new(&folder_path);
    if !path.is_dir() {
        return Err("不是有效的目录".to_string());
    }

    let mut images: Vec<ImageInfo> = Vec::new();

    let entries = fs::read_dir(path).map_err(|e| format!("读取目录失败: {}", e))?;
    for entry in entries.flatten() {
        let file_path = entry.path();
        if file_path.is_file() && is_image_file(&file_path) {
            if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                images.push(ImageInfo {
                    index: images.len(),
                    name: name.to_string(),
                    path: file_path.to_string_lossy().into_owned(),
                });
            }
        }
    }

    // 按文件名排序
    images.sort_by(|a, b| a.name.cmp(&b.name));

    // 重排 index
    for (i, img) in images.iter_mut().enumerate() {
        img.index = i;
    }

    log::info!("文件夹加载完成: {} (共 {} 张图片)", folder_path, images.len());
    Ok(FolderInfo {
        folder_path,
        images,
    })
}

// 标注文件存储路径：根据图片路径派生同名的 .annotations.json
fn annotations_path_for_image(image_path: &str) -> std::path::PathBuf {
    let p = Path::new(image_path);
    let stem = p.file_stem().unwrap_or_default().to_string_lossy();
    let parent = p.parent().unwrap_or(p);
    parent.join(format!("{}.annotations.json", stem))
}

#[tauri::command]
pub fn load_annotations_for_image(image_path: String) -> Result<Vec<Annotation>, String> {
    let ann_path = annotations_path_for_image(&image_path);
    if !ann_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&ann_path)
        .map_err(|e| format!("读取标注文件失败: {}", e))?;
    let annotations: Vec<Annotation> = serde_json::from_str(&content)
        .map_err(|e| format!("解析标注文件失败: {}", e))?;
    log::info!("标注已加载: {}", ann_path.display());
    Ok(annotations)
}

#[tauri::command]
pub fn save_annotations_for_image(image_path: String, annotations: Vec<Annotation>) -> Result<(), String> {
    let ann_path = annotations_path_for_image(&image_path);
    let json = serde_json::to_string_pretty(&annotations)
        .map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&ann_path, json).map_err(|e| format!("保存失败: {}", e))?;
    log::info!("标注已保存: {}", ann_path.display());
    Ok(())
}

// ==================== 图片加载 ====================

#[derive(serde::Serialize)]
pub struct LoadImageResult {
    pub base64: String,
    pub width: u32,
    pub height: u32,
    pub image_path: String,
}

#[tauri::command]
pub fn load_image(path: String) -> Result<LoadImageResult, String> {
    // 只读尺寸，避免整图解码 + PNG 再编码 + 巨型 base64（前端用 convertFileSrc 显示原文件）
    let (width, height) = image::image_dimensions(&path)
        .map_err(|e| format!("无法读取图像（请确认格式为常见 JPG/PNG 等）: {}", e))?;

    log::info!("图像元数据: {} ({}×{})", path, width, height);
    Ok(LoadImageResult {
        base64: String::new(),
        width,
        height,
        image_path: path,
    })
}

// ==================== 项目保存 ====================

#[tauri::command]
pub fn save_project(path: String, project: Project) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&project)
        .map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("保存失败: {}", e))?;
    log::info!("项目已保存: {}", path);
    Ok(())
}

#[tauri::command]
pub fn load_project(path: String) -> Result<Project, String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取失败: {}", e))?;
    let project: Project = serde_json::from_str(&content)
        .map_err(|e| format!("解析失败: {}", e))?;
    log::info!("项目已加载: {}", path);
    Ok(project)
}

// ==================== YOLO 导出 ====================

fn export_axis_aligned_box(b: &AxisAlignedBox) -> String {
    format!("{} {:.6} {:.6} {:.6} {:.6}", b.class_id, b.x1, b.y1, b.x2, b.y2)
}

fn export_rotated_box(rb: &RotatedBox) -> String {
    format!(
        "{} {:.6} {:.6} {:.6} {:.6} {:.6}",
        rb.class_id, rb.cx, rb.cy, rb.width, rb.height, rb.angle
    )
}

fn export_polygon(polygon: &PolygonAnnotation) -> String {
    let mut parts = vec![polygon.class_id.to_string()];
    for point in &polygon.points {
        parts.push(format!("{:.6}", point.x));
        parts.push(format!("{:.6}", point.y));
    }
    parts.join(" ")
}

#[tauri::command]
pub fn export_yolo(output_dir: String, project: Project) -> Result<(), String> {
    let path = Path::new(&output_dir);
    fs::create_dir_all(path).map_err(|e| e.to_string())?;

    let image_name = Path::new(&project.image_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let label_path = path.join(format!("{}.txt", image_name));
    let mut file = fs::File::create(&label_path).map_err(|e| e.to_string())?;

    for annotation in &project.annotations {
        let line = match annotation {
            Annotation::AxisAlignedBox(b) => export_axis_aligned_box(b),
            Annotation::RotatedBox(rb) => export_rotated_box(rb),
            Annotation::Polygon(p) => export_polygon(p),
            Annotation::Classification(_) => continue,
            Annotation::Keypoint(kp) => {
                let kp_count = project.classes
                    .iter()
                    .find(|c| c.id == kp.class_id)
                    .and_then(|c| c.keypoint_count)
                    .unwrap_or(kp.keypoints.len());

                let mut parts = vec![kp.class_id.to_string()];

                // 包围框
                if let Some(bbox) = &kp.bounding_box {
                    parts.push(format!("{:.6}", bbox.cx));
                    parts.push(format!("{:.6}", bbox.cy));
                    parts.push(format!("{:.6}", bbox.width));
                    parts.push(format!("{:.6}", bbox.height));
                } else {
                    let min_x = kp.keypoints.iter().map(|k| k.x).fold(0.0, f64::min);
                    let max_x = kp.keypoints.iter().map(|k| k.x).fold(0.0, f64::max);
                    let min_y = kp.keypoints.iter().map(|k| k.y).fold(0.0, f64::min);
                    let max_y = kp.keypoints.iter().map(|k| k.y).fold(0.0, f64::max);
                    parts.push(format!("{:.6}", (min_x + max_x) / 2.0));
                    parts.push(format!("{:.6}", (min_y + max_y) / 2.0));
                    parts.push(format!("{:.6}", max_x - min_x));
                    parts.push(format!("{:.6}", max_y - min_y));
                }

                for kp_item in &kp.keypoints {
                    parts.push(format!("{:.6}", kp_item.x));
                    parts.push(format!("{:.6}", kp_item.y));
                    parts.push(format!("{}", kp_item.visibility as i32));
                }

                for _ in kp.keypoints.len()..kp_count {
                    parts.push("-1".to_string());
                    parts.push("-1".to_string());
                    parts.push("0".to_string());
                }

                parts.join(" ")
            }
            Annotation::Ocr(ocr) => {
                if ocr.points.len() == 4 {
                    let min_x = ocr.points.iter().map(|p| p.x).fold(0.0f64, f64::min);
                    let max_x = ocr.points.iter().map(|p| p.x).fold(0.0f64, f64::max);
                    let min_y = ocr.points.iter().map(|p| p.y).fold(0.0f64, f64::min);
                    let max_y = ocr.points.iter().map(|p| p.y).fold(0.0f64, f64::max);
                    format!(
                        "{} {:.6} {:.6} {:.6} {:.6} 0 # {}",
                        ocr.class_id,
                        (min_x + max_x) / 2.0,
                        (min_y + max_y) / 2.0,
                        max_x - min_x,
                        max_y - min_y,
                        ocr.text
                    )
                } else {
                    continue;
                }
            }
        };

        if !line.is_empty() {
            writeln!(file, "{}", line).map_err(|e| e.to_string())?;
        }
    }

    // 导出分类标注
    let classification_annotations: Vec<_> = project.annotations
        .iter()
        .filter_map(|a| {
            if let Annotation::Classification(c) = a {
                Some(&c.class_ids)
            } else {
                None
            }
        })
        .collect();

    if !classification_annotations.is_empty() {
        let label_path_cls = path.join(format!("{}_classes.txt", image_name));
        let mut file_cls = fs::File::create(&label_path_cls).map_err(|e| e.to_string())?;
        let class_names: Vec<_> = classification_annotations
            .iter()
            .flat_map(|ids| ids.iter())
            .filter_map(|&id| project.classes.iter().find(|c| c.id == id))
            .map(|c| c.name.clone())
            .collect();
        for name in class_names {
            writeln!(file_cls, "{}", name).map_err(|e| e.to_string())?;
        }
    }

    log::info!("YOLO 格式已导出到: {}", output_dir);
    Ok(())
}

// ==================== 标注操作 ====================

#[tauri::command]
pub fn create_box(
    class_id: usize,
    x1: f64, y1: f64,
    x2: f64, y2: f64,
) -> Annotation {
    Annotation::AxisAlignedBox(AxisAlignedBox {
        id: Uuid::new_v4().to_string(),
        class_id,
        x1: x1.min(x2),
        y1: y1.min(y2),
        x2: x1.max(x2),
        y2: y1.max(y2),
        confidence: 1.0,
    })
}

#[tauri::command]
pub fn create_rotated_box(
    class_id: usize,
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    angle: f64,
) -> Annotation {
    let cx = (x1 + x2) / 2.0;
    let cy = (y1 + y2) / 2.0;
    let width = (x2 - x1).abs();
    let height = (y2 - y1).abs();

    Annotation::RotatedBox(RotatedBox {
        id: Uuid::new_v4().to_string(),
        class_id,
        cx,
        cy,
        width,
        height,
        angle,
        confidence: 1.0,
    })
}

#[tauri::command]
pub fn create_rotated_box2(
    class_id: usize,
    cx: f64, cy: f64,
    width: f64, height: f64,
    angle: f64,
) -> Annotation {
    Annotation::RotatedBox(RotatedBox {
        id: Uuid::new_v4().to_string(),
        class_id,
        cx,
        cy,
        width,
        height,
        angle,
        confidence: 1.0,
    })
}

#[tauri::command]
pub fn create_polygon(class_id: usize, points: Vec<Point>) -> Option<Annotation> {
    if points.len() >= 3 {
        Some(Annotation::Polygon(PolygonAnnotation {
            id: Uuid::new_v4().to_string(),
            class_id,
            points,
            holes: Vec::new(),
            confidence: 1.0,
        }))
    } else {
        None
    }
}

#[tauri::command]
pub fn create_classification(class_ids: Vec<usize>) -> Annotation {
    Annotation::Classification(ClassificationAnnotation {
        id: Uuid::new_v4().to_string(),
        class_ids,
    })
}

#[tauri::command]
pub fn create_keypoint(
    class_id: usize,
    keypoints: Vec<Keypoint>,
    bounding_box: Option<RotatedBox>,
) -> Annotation {
    Annotation::Keypoint(KeypointAnnotation {
        id: Uuid::new_v4().to_string(),
        class_id,
        keypoints,
        bounding_box,
        confidence: 1.0,
    })
}

#[tauri::command]
pub fn create_keypoint_direct(
    class_id: usize,
    keypoints: Vec<Keypoint>,
) -> Annotation {
    Annotation::Keypoint(KeypointAnnotation {
        id: Uuid::new_v4().to_string(),
        class_id,
        keypoints,
        bounding_box: None,
        confidence: 1.0,
    })
}

#[tauri::command]
pub fn create_keypoint_with_bbox(
    class_id: usize,
    keypoints: Vec<Keypoint>,
    bbox_x1: f64,
    bbox_y1: f64,
    bbox_x2: f64,
    bbox_y2: f64,
) -> Annotation {
    let cx = (bbox_x1 + bbox_x2) / 2.0;
    let cy = (bbox_y1 + bbox_y2) / 2.0;
    let width = (bbox_x2 - bbox_x1).abs();
    let height = (bbox_y2 - bbox_y1).abs();
    Annotation::Keypoint(KeypointAnnotation {
        id: Uuid::new_v4().to_string(),
        class_id,
        keypoints,
        bounding_box: Some(RotatedBox {
            id: Uuid::new_v4().to_string(),
            class_id,
            cx,
            cy,
            width,
            height,
            angle: 0.0,
            confidence: 1.0,
        }),
        confidence: 1.0,
    })
}

#[tauri::command]
pub fn create_ocr(
    class_id: usize,
    points: Vec<Point>,
    text: String,
) -> Annotation {
    Annotation::Ocr(OcrAnnotation {
        id: Uuid::new_v4().to_string(),
        class_id,
        points,
        text,
        confidence: 1.0,
    })
}

#[tauri::command]
pub fn get_default_classes() -> Vec<ClassDefinition> {
    Project::default_classes()
}

// ==================== 任务项目持久化 ====================

fn get_project_file_path(file_name: &str) -> std::path::PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("EasyLabelSlint");
    std::fs::create_dir_all(&path).ok();
    path.push(file_name);
    path
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectListData {
    pub tasks: Vec<serde_json::Value>,
}

#[tauri::command]
pub fn load_project_list(file_name: String) -> Result<serde_json::Value, String> {
    let path = get_project_file_path(&file_name);
    if !path.exists() {
        return Ok(serde_json::json!({ "tasks": [] }));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取项目文件失败: {}", e))?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析项目文件失败: {}", e))?;
    log::info!("项目列表已加载: {}", path.display());
    Ok(data)
}

#[tauri::command]
pub fn save_project_list(file_name: String, data: serde_json::Value) -> Result<(), String> {
    let path = get_project_file_path(&file_name);
    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("保存失败: {}", e))?;
    log::info!("项目列表已保存: {}", path.display());
    Ok(())
}

// ==================== 标注导出 ====================

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExportClassDef {
    pub id: usize,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExportRequest {
    pub image_folder: String,
    pub output_dir: String,
    pub task_type: String,
    pub export_format: String,
    pub classes: Vec<ExportClassDef>,
}

fn get_image_files(folder: &str) -> Result<Vec<String>, String> {
    let dir = Path::new(folder);
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let path = entry.path();
        if is_image_file(&path) {
            files.push(path.to_string_lossy().to_string());
        }
    }
    files.sort();
    Ok(files)
}

fn load_annotations(image_path: &str) -> Result<Vec<Annotation>, String> {
    let ann_path = annotations_path_for_image(image_path);
    if !ann_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&ann_path)
        .map_err(|e| format!("读取标注失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析标注失败: {}", e))
}

fn class_name_by_id(classes: &[ExportClassDef], id: usize) -> String {
    classes.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or_else(|| format!("class_{}", id))
}

/// 生成 YOLO 格式（目标检测）
fn export_yolo_detection(images: &[String], classes: &[ExportClassDef], output_dir: &str) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;

    // 生成 classes.txt
    let cls_lines: Vec<String> = classes.iter().map(|c| c.name.clone()).collect();
    fs::write(format!("{}/classes.txt", output_dir), cls_lines.join("\n"))
        .map_err(|e| format!("写入 classes.txt 失败: {}", e))?;

    for img_path in images {
        let annotations = load_annotations(img_path)?;
        let path = Path::new(img_path);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let mut lines = Vec::new();
        for ann in &annotations {
            if let Annotation::AxisAlignedBox(b) = ann {
                let x_center = (b.x1 + b.x2) / 2.0;
                let y_center = (b.y1 + b.y2) / 2.0;
                let w = (b.x2 - b.x1).abs();
                let h = (b.y2 - b.y1).abs();
                lines.push(format!("{} {:.6} {:.6} {:.6} {:.6}", b.class_id, x_center, y_center, w, h));
            }
        }
        if !lines.is_empty() {
            fs::write(format!("{}/{}.txt", output_dir, stem), lines.join("\n"))
                .map_err(|e| format!("写入 {}.txt 失败: {}", stem, e))?;
        }
    }
    Ok(std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?.join(output_dir).to_string_lossy().to_string())
}

/// 生成 YOLO OBB 格式（旋转框检测）
fn export_yolo_obb(images: &[String], classes: &[ExportClassDef], output_dir: &str) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;

    let cls_lines: Vec<String> = classes.iter().map(|c| c.name.clone()).collect();
    fs::write(format!("{}/classes.txt", output_dir), cls_lines.join("\n"))
        .map_err(|e| format!("写入 classes.txt 失败: {}", e))?;

    for img_path in images {
        let annotations = load_annotations(img_path)?;
        let path = Path::new(img_path);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let mut lines = Vec::new();
        for ann in &annotations {
            if let Annotation::RotatedBox(b) = ann {
                let angle_deg = b.angle.to_degrees();
                lines.push(format!("{} {:.6} {:.6} {:.6} {:.6} {:.2}", b.class_id, b.cx, b.cy, b.width, b.height, angle_deg));
            }
        }
        if !lines.is_empty() {
            fs::write(format!("{}/{}.txt", output_dir, stem), lines.join("\n"))
                .map_err(|e| format!("写入 {}.txt 失败: {}", stem, e))?;
        }
    }
    Ok(std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?.join(output_dir).to_string_lossy().to_string())
}

/// 生成 YOLO 格式（实例分割） — class_id x1 y1 x2 y2 ... xn yn
fn export_yolo_segmentation(images: &[String], classes: &[ExportClassDef], output_dir: &str) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;

    let cls_lines: Vec<String> = classes.iter().map(|c| c.name.clone()).collect();
    fs::write(format!("{}/classes.txt", output_dir), cls_lines.join("\n"))
        .map_err(|e| format!("写入 classes.txt 失败: {}", e))?;

    for img_path in images {
        let annotations = load_annotations(img_path)?;
        let path = Path::new(img_path);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let mut lines = Vec::new();
        for ann in &annotations {
            if let Annotation::Polygon(p) = ann {
                let coords: Vec<String> = p.points.iter().map(|pt| format!("{:.6}", pt.x)).collect();
                let coords_y: Vec<String> = p.points.iter().map(|pt| format!("{:.6}", pt.y)).collect();
                let all: Vec<String> = coords.into_iter().zip(coords_y.into_iter()).flat_map(|(x, y)| vec![x, y]).collect();
                lines.push(format!("{} {}", p.class_id, all.join(" ")));
            }
        }
        if !lines.is_empty() {
            fs::write(format!("{}/{}.txt", output_dir, stem), lines.join("\n"))
                .map_err(|e| format!("写入 {}.txt 失败: {}", stem, e))?;
        }
    }
    Ok(std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?.join(output_dir).to_string_lossy().to_string())
}

/// 生成 YOLO 格式（分类） — class_id（每张图片一个文件）
fn export_yolo_classification(images: &[String], classes: &[ExportClassDef], output_dir: &str) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;

    let cls_lines: Vec<String> = classes.iter().map(|c| c.name.clone()).collect();
    fs::write(format!("{}/classes.txt", output_dir), cls_lines.join("\n"))
        .map_err(|e| format!("写入 classes.txt 失败: {}", e))?;

    for img_path in images {
        let annotations = load_annotations(img_path)?;
        let path = Path::new(img_path);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let mut class_ids = Vec::new();
        for ann in &annotations {
            if let Annotation::Classification(c) = ann {
                for &cid in &c.class_ids {
                    class_ids.push(cid);
                }
            }
        }
        if !class_ids.is_empty() {
            fs::write(format!("{}/{}.txt", output_dir, stem), class_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join("\n"))
                .map_err(|e| format!("写入 {}.txt 失败: {}", stem, e))?;
        }
    }
    Ok(std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?.join(output_dir).to_string_lossy().to_string())
}

/// 生成 COCO JSON 格式（实例分割/关键点）
fn export_coco_json(images: &[String], classes: &[ExportClassDef], task_type: &str, output_dir: &str) -> Result<String, String> {
    use std::collections::HashMap;

    let mut coco_images = Vec::new();
    let mut coco_annotations = Vec::new();
    let mut coco_categories = Vec::new();
    let mut ann_id: i64 = 1;

    for (img_idx, img_path) in images.iter().enumerate() {
        let path = Path::new(img_path);
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let annotations = load_annotations(img_path)?;

        coco_images.push(serde_json::json!({
            "id": img_idx + 1,
            "file_name": file_name,
            "width": 0,
            "height": 0,
        }));

        for ann in &annotations {
            let (class_id, segmentation, bbox) = match ann {
                Annotation::Polygon(p) => {
                    let seg: Vec<f64> = p.points.iter().flat_map(|pt| vec![pt.x, pt.y]).collect();
                    let xs: Vec<f64> = p.points.iter().map(|pt| pt.x).collect();
                    let ys: Vec<f64> = p.points.iter().map(|pt| pt.y).collect();
                    let x1 = xs.iter().cloned().fold(f64::INFINITY, f64::min);
                    let y1 = ys.iter().cloned().fold(f64::INFINITY, f64::min);
                    let x2 = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let y2 = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    (p.class_id, Some(seg), vec![x1, y1, x2 - x1, y2 - y1])
                }
                Annotation::AxisAlignedBox(b) => {
                    (b.class_id, None, vec![b.x1, b.y1, b.x2 - b.x1, b.y2 - b.y1])
                }
                _ => continue,
            };

            let mut ann_obj = serde_json::json!({
                "id": ann_id,
                "image_id": img_idx + 1,
                "category_id": class_id + 1,
                "bbox": bbox,
                "area": bbox[2] * bbox[3],
                "iscrowd": 0,
            });
            if let Some(seg) = segmentation {
                ann_obj["segmentation"] = serde_json::json!([seg]);
            }
            coco_annotations.push(ann_obj);
            ann_id += 1;
        }
    }

    for cls in classes {
        coco_categories.push(serde_json::json!({
            "id": cls.id + 1,
            "name": cls.name,
            "supercategory": "object",
        }));
    }

    let coco = serde_json::json!({
        "images": coco_images,
        "annotations": coco_annotations,
        "categories": coco_categories,
    });

    let output_dir = if output_dir.is_empty() { if task_type == "segmentation" { "coco_seg_export" } else { "coco_kp_export" } } else { output_dir };
    fs::create_dir_all(output_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;
    let json = serde_json::to_string_pretty(&coco).map_err(|e| format!("序列化COCO JSON失败: {}", e))?;
    fs::write(format!("{}/annotations.json", output_dir), &json)
        .map_err(|e| format!("写入 annotations.json 失败: {}", e))?;

    Ok(std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?.join(output_dir).to_string_lossy().to_string())
}

/// 生成 PaddleOCR 格式
fn export_paddleocr(images: &[String], classes: &[ExportClassDef], output_dir: &str) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;

    for img_path in images {
        let annotations = load_annotations(img_path)?;
        let path = Path::new(img_path);
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let mut lines = Vec::new();
        for ann in &annotations {
            if let Annotation::Ocr(o) = ann {
                let pts: Vec<String> = o.points.iter().map(|p| format!("[{:.6},{:.6}]", p.x, p.y)).collect();
                let transcription = if o.text.is_empty() { "###" } else { &o.text };
                lines.push(format!("[{}]\t{}", pts.join(","), transcription));
            }
        }
        if !lines.is_empty() {
            fs::write(format!("{}/{}_ocr.txt", output_dir, file_name), lines.join("\n"))
                .map_err(|e| format!("写入 OCR 文件失败: {}", e))?;
        }
    }
    Ok(std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?.join(output_dir).to_string_lossy().to_string())
}

/// 生成 CSV 格式（分类）
fn export_classification_csv(images: &[String], classes: &[ExportClassDef], output_dir: &str) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;

    let mut csv_lines = Vec::new();
    csv_lines.push("image_name,class_id,class_name".to_string());

    for img_path in images {
        let annotations = load_annotations(img_path)?;
        let path = Path::new(img_path);
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        for ann in &annotations {
            if let Annotation::Classification(c) = ann {
                for &cid in &c.class_ids {
                    let cname = class_name_by_id(classes, cid);
                    csv_lines.push(format!("{},{},{}", file_name, cid, cname));
                }
            }
        }
    }

    fs::write(format!("{}/classification.csv", output_dir), csv_lines.join("\n"))
        .map_err(|e| format!("写入 CSV 失败: {}", e))?;

    Ok(std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?.join(output_dir).to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_annotations(request: ExportRequest) -> Result<String, String> {
    let images = get_image_files(&request.image_folder)?;
    if images.is_empty() {
        return Err("没有找到图片文件".to_string());
    }

    match (request.task_type.as_str(), request.export_format.as_str()) {
        ("detection", "yolo") => export_yolo_detection(&images, &request.classes, &request.output_dir),
        ("rotated_detection", "yolo_obb") => export_yolo_obb(&images, &request.classes, &request.output_dir),
        ("segmentation", "yolo") => export_yolo_segmentation(&images, &request.classes, &request.output_dir),
        ("segmentation", "coco_json") | ("keypoint", "coco_json") => export_coco_json(&images, &request.classes, &request.task_type, &request.output_dir),
        ("classification", "yolo") => export_yolo_classification(&images, &request.classes, &request.output_dir),
        ("classification", "csv") => export_classification_csv(&images, &request.classes, &request.output_dir),
        ("ocr", "paddleocr") => export_paddleocr(&images, &request.classes, &request.output_dir),
        _ => Err(format!("不支持的任务类型 \"{}\" 与导出格式 \"{}\" 组合", request.task_type, request.export_format)),
    }
}

// ==================== 设置持久化 ====================

fn get_settings_path() -> std::path::PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("EasyLabelSlint");
    std::fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

#[tauri::command]
pub fn load_settings() -> AppSettings {
    let path = get_settings_path();
    if !path.exists() {
        return AppSettings::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("保存设置失败: {}", e))?;
    log::info!("设置已保存: {}", path.display());
    Ok(())
}
