use std::path::PathBuf;
use crate::models::{ImageItem, ImageSource};
use uuid::Uuid;
use image::GenericImageView;

pub fn create_image_item_from_path(path: PathBuf) -> ImageItem {
    let metadata = std::fs::metadata(&path).ok();
    let timestamp = metadata
        .and_then(|m| m.modified().ok())
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    // Get image dimensions
    let (width, height) = get_image_dimensions(&path);

    ImageItem {
        id: Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        thumbnail: String::new(), // Will be generated on frontend
        width,
        height,
        timestamp,
        source: ImageSource::Screenshot,
        is_pinned: false,
        pin_hwnd: None,
    }
}

fn get_image_dimensions(path: &PathBuf) -> (u32, u32) {
    if let Ok(img) = image::open(path) {
        let (w, h) = img.dimensions();
        (w, h)
    } else {
        (0, 0)
    }
}

pub fn is_image_file(path: &PathBuf) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "gif" | "tiff" | "tif")
    } else {
        false
    }
}

pub fn start_file_watcher(_state: std::sync::Arc<crate::store::AppState>, _path: PathBuf) {
    // TODO: Implement file watcher with notify crate
    // This would monitor the screenshots folder for new images
    println!("File watcher started for path: {:?}", _path);
}
