#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use easy_label_lib::commands::{*, create_keypoint, create_ocr, create_keypoint_with_bbox, load_project_list, save_project_list, load_settings, save_settings, export_annotations};
use tauri::Manager;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("EasyLabel 启动中...");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_image,
            save_project,
            load_project,
            export_yolo,
            create_rotated_box,
            create_rotated_box2,
            create_box,
            create_polygon,
            create_classification,
            create_keypoint,
            create_keypoint_with_bbox,
            create_ocr,
            load_project_list,
            save_project_list,
            get_default_classes,
            load_folder,
            load_annotations_for_image,
            save_annotations_for_image,
            load_settings,
            save_settings,
            export_annotations,
            get_annotation_statuses,
            auto_annotate,
        ])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
