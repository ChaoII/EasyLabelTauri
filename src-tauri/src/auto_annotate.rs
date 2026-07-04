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
        let (img_w, img_h) = (img.width() as f64, img.height() as f64);
        let results = model.predict(&img).map_err(|e| format!("检测推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for det in &results {
            let class_id = det.label_id as usize;
            if class_id >= classes.len() { continue; }
            // 归一化坐标到 0-1 范围
            let x1 = det.rect.x as f64 / img_w;
            let y1 = det.rect.y as f64 / img_h;
            let x2 = (det.rect.x as f64 + det.rect.width as f64) / img_w;
            let y2 = (det.rect.y as f64 + det.rect.height as f64) / img_h;
            annotations.push(Annotation::AxisAlignedBox(AxisAlignedBox {
                id: uuid::Uuid::new_v4().to_string(),
                class_id,
                x1,
                y1,
                x2,
                y2,
                confidence: det.score as f64,
                locked: false,
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
        let (img_w, img_h) = (img.width() as f64, img.height() as f64);
        let results = model.predict(&img).map_err(|e| format!("OBB推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for obb in &results {
            let class_id = obb.label_id as usize;
            if class_id >= classes.len() { continue; }
            annotations.push(Annotation::RotatedBox(RotatedBox {
                id: uuid::Uuid::new_v4().to_string(),
                class_id,
                cx: obb.xc as f64 / img_w,
                cy: obb.yc as f64 / img_h,
                width: obb.width as f64 / img_w,
                height: obb.height as f64 / img_h,
                angle: obb.angle as f64,
                confidence: obb.score as f64,
                locked: false,
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
                extract_polygon_from_mask(&seg.mask_buffer, seg.mask_shape[0] as usize, seg.mask_shape[1] as usize)
            } else {
                // fallback: 用检测框的四个角点 (归一化)
                vec![
                    Point { x: seg.rect.x as f64 / img_w, y: seg.rect.y as f64 / img_h },
                    Point { x: (seg.rect.x + seg.rect.width) as f64 / img_w, y: seg.rect.y as f64 / img_h },
                    Point { x: (seg.rect.x + seg.rect.width) as f64 / img_w, y: (seg.rect.y + seg.rect.height) as f64 / img_h },
                    Point { x: seg.rect.x as f64 / img_w, y: (seg.rect.y + seg.rect.height) as f64 / img_h },
                ]
            };

            annotations.push(Annotation::Polygon(PolygonAnnotation {
                id: uuid::Uuid::new_v4().to_string(),
                class_id,
                points,
                holes: vec![],
                confidence: seg.score as f64,
                locked: false,
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
pub fn extract_polygon_from_mask(mask: &[u8], mask_h: usize, mask_w: usize) -> Vec<Point> {
    let mut points = Vec::new();
    // 采样边界点: 每隔几步取一个点
    let step = (mask_h.max(mask_w) / 32).max(1);
    // 上边
    for x in (0..mask_w).step_by(step) {
        for y in 0..mask_h {
            if mask[y * mask_w + x] > 0 {
                points.push(Point { x: x as f64 / mask_w as f64, y: y as f64 / mask_h as f64 });
                break;
            }
        }
    }
    // 下边
    for x in (0..mask_w).step_by(step) {
        for y in (0..mask_h).rev() {
            if mask[y * mask_w + x] > 0 {
                points.push(Point { x: x as f64 / mask_w as f64, y: y as f64 / mask_h as f64 });
                break;
            }
        }
    }
    // 去重
    points.dedup_by(|a, b| (a.x - b.x).abs() < 1e-4 && (a.y - b.y).abs() < 1e-4);
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
        let (img_w, img_h) = (img.width() as f64, img.height() as f64);
        let results = model.predict(&img).map_err(|e| format!("关键点推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for pose in &results {
            let class_id = 0usize;

            let keypoints: Vec<Keypoint> = pose.keypoints.iter().enumerate().map(|(i, kp)| {
                let name = format!("kp_{}", i);
                Keypoint {
                    x: kp.x as f64 / img_w,
                    y: kp.y as f64 / img_h,
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
                    cx: (pose.rect.x as f64 + pose.rect.width as f64 / 2.0) / img_w,
                    cy: (pose.rect.y as f64 + pose.rect.height as f64 / 2.0) / img_h,
                    width: pose.rect.width as f64 / img_w,
                    height: pose.rect.height as f64 / img_h,
                    angle: 0.0,
                    confidence: pose.score as f64,
                    locked: false,
                }),
                keypoints,
                confidence: pose.score as f64,
                locked: false,
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
                locked: false,
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
        let (img_w, img_h) = (img.width() as f64, img.height() as f64);
        let results = ocr_model.predict(&img).map_err(|e| format!("OCR推理失败: {}", e))?;

        let mut annotations = Vec::new();
        for ocr in &results {
            // PaddleOCR 返回的points是4个点的多边形
            let points: Vec<Point> = ocr.points.iter().map(|&(px, py)| Point {
                x: px as f64 / img_w,
                y: py as f64 / img_h,
            }).collect();

            if points.len() < 4 { continue; }

            annotations.push(Annotation::Ocr(OcrAnnotation {
                id: uuid::Uuid::new_v4().to_string(),
                class_id: 0,
                points,
                text: ocr.text.clone(),
                confidence: ocr.score as f64,
                locked: false,
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

/// 评估结果
#[derive(serde::Serialize)]
pub struct EvalMetrics {
    pub total_images: usize,
    pub total_gt: usize,
    pub total_pred: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub per_image: Vec<EvalImageResult>,
}

#[derive(serde::Serialize)]
pub struct EvalImageResult {
    pub image_name: String,
    pub gt_count: usize,
    pub pred_count: usize,
    pub tp: usize,
    pub fp: usize,
    pub fn_count: usize,
}

fn iou(a: &AxisAlignedBox, bx: f64, by: f64, bw: f64, bh: f64) -> f64 {
    let ax1 = a.x1; let ay1 = a.y1; let ax2 = a.x2; let ay2 = a.y2;
    let bx1 = bx; let by1 = by; let bx2 = bx + bw; let by2 = by + bh;
    let ix1 = ax1.max(bx1); let iy1 = ay1.max(by1);
    let ix2 = ax2.min(bx2); let iy2 = ay2.min(by2);
    let iw = (ix2 - ix1).max(0.0); let ih = (iy2 - iy1).max(0.0);
    let inter = iw * ih;
    let union = (ax2 - ax1) * (ay2 - ay1) + bw * bh - inter;
    if union <= 0.0 { 0.0 } else { inter / union }
}

/// 模型评估: 在标注数据上运行推理并对比
pub fn evaluate_detection(
    images: &[String],
    model_path: &str,
    classes: &[ExportClassDef],
) -> Result<EvalMetrics, String> {
    let opt = default_runtime();
    let model = modeldeploy::vision::detection::UltralyticsDet::new(model_path, &opt)
        .map_err(|e| format!("加载检测模型失败: {}", e))?;

    let mut total_gt = 0usize;
    let mut total_pred = 0usize;
    let mut total_tp = 0usize;
    let mut total_fp = 0usize;
    let mut total_fn = 0usize;
    let mut per_image = Vec::new();

    for img_path in images {
        let p = std::path::Path::new(img_path);
        let image_name = p.file_name().unwrap_or_default().to_string_lossy().to_string();

        // 加载真实标注
        let ann_path = annotations_path_for_image(img_path);
        let gt_annotations: Vec<AxisAlignedBox> = if ann_path.exists() {
            std::fs::read_to_string(&ann_path).ok()
                .and_then(|c| serde_json::from_str::<Vec<Annotation>>(&c).ok())
                .map(|anns| anns.into_iter().filter_map(|a| {
                    if let Annotation::AxisAlignedBox(b) = a { Some(b) } else { None }
                }).collect())
                .unwrap_or_default()
        } else { continue; };

        if gt_annotations.is_empty() { continue; }

        // 运行推理
        let img = load_md_image(img_path)?;
        let (img_w, img_h) = (img.width() as f64, img.height() as f64);
        let results = match model.predict(&img) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let predictions: Vec<(f64, f64, f64, f64)> = results.iter().map(|det| {
            (det.rect.x as f64 / img_w, det.rect.y as f64 / img_h,
             det.rect.width as f64 / img_w, det.rect.height as f64 / img_h)
        }).collect();

        // 计算 TP/FP/FN
        let mut gt_matched = vec![false; gt_annotations.len()];
        let mut pred_matched = vec![false; predictions.len()];

        for (gi, gt) in gt_annotations.iter().enumerate() {
            for (pi, pred) in predictions.iter().enumerate() {
                if iou(gt, pred.0, pred.1, pred.2, pred.3) > 0.5 {
                    gt_matched[gi] = true;
                    pred_matched[pi] = true;
                }
            }
        }

        let tp = gt_matched.iter().filter(|&&m| m).count();
        let fp = pred_matched.iter().filter(|&&m| !m).count();
        let fn_count = gt_matched.iter().filter(|&&m| !m).count();

        total_gt += gt_annotations.len();
        total_pred += predictions.len();
        total_tp += tp;
        total_fp += fp;
        total_fn += fn_count;

        per_image.push(EvalImageResult {
            image_name,
            gt_count: gt_annotations.len(),
            pred_count: predictions.len(),
            tp, fp, fn_count,
        });
    }

    let precision = if total_tp + total_fp > 0 { total_tp as f64 / (total_tp + total_fp) as f64 } else { 0.0 };
    let recall = if total_tp + total_fn > 0 { total_tp as f64 / (total_tp + total_fn) as f64 } else { 0.0 };
    let f1 = if precision + recall > 0.0 { 2.0 * precision * recall / (precision + recall) } else { 0.0 };

    Ok(EvalMetrics {
        total_images: per_image.len(),
        total_gt, total_pred, true_positives: total_tp,
        false_positives: total_fp, false_negatives: total_fn,
        precision, recall, f1, per_image,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iou_perfect_overlap() {
        let a = AxisAlignedBox {
            id: "a".into(), class_id: 0,
            x1: 0.0, y1: 0.0, x2: 1.0, y2: 1.0,
            confidence: 1.0, locked: false,
        };
        let iou_val = iou(&a, 0.0, 0.0, 1.0, 1.0);
        assert!((iou_val - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_iou_no_overlap() {
        let a = AxisAlignedBox {
            id: "a".into(), class_id: 0,
            x1: 0.0, y1: 0.0, x2: 0.5, y2: 0.5,
            confidence: 1.0, locked: false,
        };
        let iou_val = iou(&a, 0.6, 0.6, 0.3, 0.3);
        assert!((iou_val - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_iou_partial() {
        let a = AxisAlignedBox {
            id: "a".into(), class_id: 0,
            x1: 0.0, y1: 0.0, x2: 0.6, y2: 0.6,
            confidence: 1.0, locked: false,
        };
        let iou_val = iou(&a, 0.4, 0.4, 0.6, 0.6);
        // intersection: [0.4,0.4] to [0.6,0.6] = 0.2*0.2 = 0.04
        // union: 0.36 + 0.36 - 0.04 = 0.68
        // iou: 0.04/0.68 ≈ 0.0588
        assert!((iou_val - 0.04 / 0.68).abs() < 1e-4);
    }

    #[test]
    fn test_extract_polygon_from_mask_empty() {
        let mask = vec![0u8; 64 * 64];
        let points = extract_polygon_from_mask(&mask, 64, 64);
        assert!(points.is_empty());
    }

    #[test]
    fn test_extract_polygon_from_mask_full() {
        let mask = vec![255u8; 32 * 32];
        let points = extract_polygon_from_mask(&mask, 32, 32);
        assert!(!points.is_empty());
        for p in &points {
            assert!(p.x >= 0.0 && p.x <= 1.0);
            assert!(p.y >= 0.0 && p.y <= 1.0);
        }
    }
}
