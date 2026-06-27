use crate::models::*;
use image::GenericImageView;
use modeldeploy::image::Image;
use modeldeploy::runtime::RuntimeOption;
use std::path::Path;

/// 自动标注配置
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AutoAnnotateRequest {
    pub image_folder: String,
    pub task_type: String,
    pub classes: Vec<ExportClassDef>,
    /// 模型路径 (检测/旋转框/分割/关键点/分类)
    pub model_path: Option<String>,
    /// OCR 模型路径 (JSON 格式: {"det": "...", "cls": "...", "rec": "...", "dict": "..."})
    pub ocr_models: Option<String>,
    /// 仅标注当前图片 (为 None 时标注全部)
    pub current_image: Option<String>,
}

#[derive(serde::Serialize)]
pub struct AutoAnnotateResult {
    pub total_images: usize,
    pub annotated_images: usize,
    pub total_annotations: usize,
}

/// 运行时选项
pub fn default_runtime() -> RuntimeOption {
    RuntimeOption::new()
        .gpu(0)
        .fp16(true)
        .ort_backend()
}

/// 图片路径转 modeldeploy Image
pub fn load_md_image(path: &str) -> Result<modeldeploy::image::Image, String> {
    modeldeploy::image::Image::read(path).map_err(|e| format!("加载图片失败: {}", e))
}

/// 检测标注: YOLO Detection → AxisAlignedBox
pub fn auto_detect(
    images: &[String],
    model_path: &str,
    output_dir: &str,
    classes: &[ExportClassDef],
) -> Result<AutoAnnotateResult, String> {
    let opt = default_runtime();
    let model = modeldeploy::vision::detection::UltralyticsDet::new(model_path, &opt)
        .map_err(|e| format!("加载检测模型失败: {}", e))?;

    let mut total_anns = 0usize;
    for img_path in images {
        let img = load_md_image(img_path)?;
        let results = model.predict(&img).map_err(|e| format!("检测推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for det in &results {
            let class_id = det.label_id as usize;
            if class_id >= classes.len() { continue; }
            annotations.push(Annotation::AxisAlignedBox(AxisAlignedBox {
                id: uuid::Uuid::new_v4().to_string(),
                class_id,
                x1: det.rect.x as f64,
                y1: det.rect.y as f64,
                x2: (det.rect.x + det.rect.width) as f64,
                y2: (det.rect.y + det.rect.height) as f64,
                confidence: det.score as f64,
            }));
        }

        save_annotations_for_image_internal(img_path, &annotations)?;
        total_anns += annotations.len();
    }

    Ok(AutoAnnotateResult {
        total_images: images.len(),
        annotated_images: images.iter().filter(|p| {
            let ann_path = annotations_path_for_image(p);
            ann_path.exists() && std::fs::metadata(&ann_path).map(|m| m.len() > 10).unwrap_or(false)
        }).count(),
        total_annotations: total_anns,
    })
}

/// 旋转框标注: YOLO OBB → RotatedBox
pub fn auto_obb(
    images: &[String],
    model_path: &str,
    output_dir: &str,
    classes: &[ExportClassDef],
) -> Result<AutoAnnotateResult, String> {
    let opt = default_runtime();
    let model = modeldeploy::vision::obb_iseg_pose::UltralyticsObb::new(model_path, &opt)
        .map_err(|e| format!("加载OBB模型失败: {}", e))?;

    let mut total_anns = 0usize;
    for img_path in images {
        let img = load_md_image(img_path)?;
        let results = model.predict(&img).map_err(|e| format!("OBB推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for obb in &results {
            let class_id = obb.label_id as usize;
            if class_id >= classes.len() { continue; }
            annotations.push(Annotation::RotatedBox(RotatedBox {
                id: uuid::Uuid::new_v4().to_string(),
                class_id,
                cx: obb.xc as f64,
                cy: obb.yc as f64,
                width: obb.width as f64,
                height: obb.height as f64,
                angle: obb.angle as f64,
                confidence: obb.score as f64,
            }));
        }

        save_annotations_for_image_internal(img_path, &annotations)?;
        total_anns += annotations.len();
    }

    Ok(AutoAnnotateResult {
        total_images: images.len(),
        annotated_images: images.iter().filter(|p| {
            let ann_path = annotations_path_for_image(p);
            ann_path.exists() && std::fs::metadata(&ann_path).map(|m| m.len() > 10).unwrap_or(false)
        }).count(),
        total_annotations: total_anns,
    })
}

/// 实例分割标注: YOLO Seg → PolygonAnnotation (从mask提取轮廓)
pub fn auto_segmentation(
    images: &[String],
    model_path: &str,
    output_dir: &str,
    classes: &[ExportClassDef],
) -> Result<AutoAnnotateResult, String> {
    let opt = default_runtime();
    let model = modeldeploy::vision::obb_iseg_pose::UltralyticsSeg::new(model_path, &opt)
        .map_err(|e| format!("加载分割模型失败: {}", e))?;

    let mut total_anns = 0usize;
    for img_path in images {
        let img = load_md_image(img_path)?;
        let (img_w, img_h) = (img.width() as f64, img.height() as f64);
        let results = model.predict(&img).map_err(|e| format!("分割推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for seg in &results {
            let class_id = seg.label_id as usize;
            if class_id >= classes.len() { continue; }

            // 从mask提取多边形轮廓
            let points = if !seg.mask_buffer.is_empty() && seg.mask_shape.len() >= 2 {
                extract_polygon_from_mask(&seg.mask_buffer, seg.mask_shape[0] as usize, seg.mask_shape[1] as usize, img_w, img_h)
            } else {
                // fallback: 用检测框的四个角点
                vec![
                    Point { x: seg.rect.x as f64, y: seg.rect.y as f64 },
                    Point { x: (seg.rect.x + seg.rect.width) as f64, y: seg.rect.y as f64 },
                    Point { x: (seg.rect.x + seg.rect.width) as f64, y: (seg.rect.y + seg.rect.height) as f64 },
                    Point { x: seg.rect.x as f64, y: (seg.rect.y + seg.rect.height) as f64 },
                ]
            };

            annotations.push(Annotation::Polygon(PolygonAnnotation {
                id: uuid::Uuid::new_v4().to_string(),
                class_id,
                points,
                holes: vec![],
                confidence: seg.score as f64,
            }));
        }

        save_annotations_for_image_internal(img_path, &annotations)?;
        total_anns += annotations.len();
    }

    Ok(AutoAnnotateResult {
        total_images: images.len(),
        annotated_images: images.iter().filter(|p| {
            let ann_path = annotations_path_for_image(p);
            ann_path.exists() && std::fs::metadata(&ann_path).map(|m| m.len() > 10).unwrap_or(false)
        }).count(),
        total_annotations: total_anns,
    })
}

/// 从二值mask提取多边形轮廓点 (简化版: 取边界点)
pub fn extract_polygon_from_mask(mask: &[u8], mask_h: usize, mask_w: usize, img_w: f64, img_h: f64) -> Vec<Point> {
    let mut points = Vec::new();
    // 采样边界点: 每隔几步取一个点
    let step = (mask_h.max(mask_w) / 32).max(1);
    // 上边
    for x in (0..mask_w).step_by(step) {
        for y in 0..mask_h {
            if mask[y * mask_w + x] > 0 {
                points.push(Point { x: x as f64 / mask_w as f64 * img_w, y: y as f64 / mask_h as f64 * img_h });
                break;
            }
        }
    }
    // 下边
    for x in (0..mask_w).step_by(step) {
        for y in (0..mask_h).rev() {
            if mask[y * mask_w + x] > 0 {
                points.push(Point { x: x as f64 / mask_w as f64 * img_w, y: y as f64 / mask_h as f64 * img_h });
                break;
            }
        }
    }
    // 去重
    points.dedup_by(|a, b| (a.x - b.x).abs() < 1.0 && (a.y - b.y).abs() < 1.0);
    points
}

/// 关键点标注: YOLO Pose → KeypointAnnotation
pub fn auto_keypoint(
    images: &[String],
    model_path: &str,
    output_dir: &str,
    classes: &[ExportClassDef],
) -> Result<AutoAnnotateResult, String> {
    let opt = default_runtime();
    let model = modeldeploy::vision::obb_iseg_pose::UltralyticsPose::new(model_path, &opt)
        .map_err(|e| format!("加载关键点模型失败: {}", e))?;

    let mut total_anns = 0usize;
    for img_path in images {
        let img = load_md_image(img_path)?;
        let results = model.predict(&img).map_err(|e| format!("关键点推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for pose in &results {
            let class_id = 0usize;

            let keypoints: Vec<Keypoint> = pose.keypoints.iter().enumerate().map(|(i, kp)| {
                let name = format!("kp_{}", i);
                Keypoint {
                    x: kp.x as f64,
                    y: kp.y as f64,
                    visibility: if kp.x > 0.0 && kp.y > 0.0 { Visibility::Visible } else { Visibility::Hidden },
                    name,
                }
            }).collect();

            annotations.push(Annotation::Keypoint(KeypointAnnotation {
                id: uuid::Uuid::new_v4().to_string(),
                class_id,
                bounding_box: Some(RotatedBox {
                    id: uuid::Uuid::new_v4().to_string(),
                    class_id,
                    cx: (pose.rect.x as f64 + pose.rect.width as f64 / 2.0) as f64,
                    cy: (pose.rect.y as f64 + pose.rect.height as f64 / 2.0) as f64,
                    width: pose.rect.width as f64,
                    height: pose.rect.height as f64,
                    angle: 0.0,
                    confidence: pose.score as f64,
                }),
                keypoints,
                confidence: pose.score as f64,
            }));
        }

        save_annotations_for_image_internal(img_path, &annotations)?;
        total_anns += annotations.len();
    }

    Ok(AutoAnnotateResult {
        total_images: images.len(),
        annotated_images: images.iter().filter(|p| {
            let ann_path = annotations_path_for_image(p);
            ann_path.exists() && std::fs::metadata(&ann_path).map(|m| m.len() > 10).unwrap_or(false)
        }).count(),
        total_annotations: total_anns,
    })
}

/// 分类标注: YOLO Cls → ClassificationAnnotation
pub fn auto_classification(
    images: &[String],
    model_path: &str,
    output_dir: &str,
    classes: &[ExportClassDef],
) -> Result<AutoAnnotateResult, String> {
    let opt = default_runtime();
    let model = modeldeploy::vision::classification::UltralyticsCls::new(model_path, &opt)
        .map_err(|e| format!("加载分类模型失败: {}", e))?;

    let mut total_anns = 0usize;
    for img_path in images {
        let img = load_md_image(img_path)?;
        let results = model.predict(&img, 5).map_err(|e| format!("分类推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for cls in &results {
            let class_id = cls.label_id as usize;
            if class_id >= classes.len() { continue; }
            // 只保留score > 0.3的结果
            if cls.score < 0.3 { continue; }
            annotations.push(Annotation::Classification(ClassificationAnnotation {
                id: uuid::Uuid::new_v4().to_string(),
                class_ids: vec![class_id],
            }));
        }

        if !annotations.is_empty() {
            save_annotations_for_image_internal(img_path, &annotations)?;
            total_anns += annotations.len();
        }
    }

    Ok(AutoAnnotateResult {
        total_images: images.len(),
        annotated_images: images.iter().filter(|p| {
            let ann_path = annotations_path_for_image(p);
            ann_path.exists() && std::fs::metadata(&ann_path).map(|m| m.len() > 10).unwrap_or(false)
        }).count(),
        total_annotations: total_anns,
    })
}

/// OCR标注: PaddleOCR → OcrAnnotation
pub fn auto_ocr(
    images: &[String],
    ocr_models_json: &str,
    output_dir: &str,
    classes: &[ExportClassDef],
) -> Result<AutoAnnotateResult, String> {
    let models: serde_json::Value = serde_json::from_str(ocr_models_json)
        .map_err(|e| format!("解析OCR模型配置失败: {}", e))?;

    let det_path = models["det"].as_str().ok_or("缺少 det 模型路径")?;
    let cls_path = models["cls"].as_str().ok_or("缺少 cls 模型路径")?;
    let rec_path = models["rec"].as_str().ok_or("缺少 rec 模型路径")?;
    let dict_path = models["dict"].as_str().ok_or("缺少 dict 路径")?;

    let opt = default_runtime();
    let ocr_model = modeldeploy::vision::ocr_lpr_attr::PaddleOcr::new(
        det_path, cls_path, rec_path, dict_path, &opt,
    ).map_err(|e| format!("加载OCR模型失败: {}", e))?;

    let mut total_anns = 0usize;
    for img_path in images {
        let img = load_md_image(img_path)?;
        let results = ocr_model.predict(&img).map_err(|e| format!("OCR推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for ocr in &results {
            // PaddleOCR 返回的points是4个点的多边形
            let points: Vec<Point> = ocr.points.iter().map(|&(px, py)| Point {
                x: px as f64,
                y: py as f64,
            }).collect();

            if points.len() < 4 { continue; }

            annotations.push(Annotation::Ocr(OcrAnnotation {
                id: uuid::Uuid::new_v4().to_string(),
                class_id: 0,
                points,
                text: ocr.text.clone(),
                confidence: ocr.score as f64,
            }));
        }

        save_annotations_for_image_internal(img_path, &annotations)?;
        total_anns += annotations.len();
    }

    Ok(AutoAnnotateResult {
        total_images: images.len(),
        annotated_images: images.iter().filter(|p| {
            let ann_path = annotations_path_for_image(p);
            ann_path.exists() && std::fs::metadata(&ann_path).map(|m| m.len() > 10).unwrap_or(false)
        }).count(),
        total_annotations: total_anns,
    })
}

/// 保存标注到文件
pub fn save_annotations_for_image_internal(image_path: &str, annotations: &[Annotation]) -> Result<(), String> {
    let ann_path = annotations_path_for_image(image_path);
    let json = serde_json::to_string_pretty(annotations)
        .map_err(|e| format!("序列化标注失败: {}", e))?;
    std::fs::write(&ann_path, json)
        .map_err(|e| format!("保存标注文件失败: {}", e))?;
    Ok(())
}

/// 标注文件路径
pub fn annotations_path_for_image(image_path: &str) -> std::path::PathBuf {
    let p = Path::new(image_path);
    let stem = p.file_stem().unwrap_or_default().to_string_lossy();
    let parent = p.parent().unwrap_or(p);
    parent.join(format!("{}.annotations.json", stem))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExportClassDef {
    pub id: usize,
    pub name: String,
}
