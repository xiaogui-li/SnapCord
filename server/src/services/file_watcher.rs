use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use base64::Engine;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::Emitter;
use uuid::Uuid;

use crate::models::{ImageItem, ImageSource};
use crate::store::AppState;

const THUMB_MAX: u32 = 480;

fn generate_thumbnail(path: &Path) -> String {
    let Ok(img) = image::open(path) else {
        return String::new();
    };
    let thumb = img.thumbnail(THUMB_MAX, THUMB_MAX);
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    if thumb
        .write_to(&mut cursor, image::ImageFormat::Jpeg)
        .is_err()
    {
        return String::new();
    }
    format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&buf)
    )
}

pub fn create_image_item_from_path(path: PathBuf) -> ImageItem {
    let metadata = std::fs::metadata(&path).ok();
    let timestamp = metadata
        .and_then(|m| m.modified().ok())
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let (width, height) = get_image_dimensions(&path);
    let thumbnail = generate_thumbnail(&path);

    ImageItem {
        id: Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        thumbnail,
        width,
        height,
        timestamp,
        source: ImageSource::Screenshot,
        is_pinned: false,
        pin_hwnd: None,
    }
}

fn get_image_dimensions(path: &Path) -> (u32, u32) {
    use image::GenericImageView;
    match image::open(path) {
        Ok(img) => img.dimensions(),
        Err(_) => (0, 0),
    }
}

pub fn is_image_file(path: &Path) -> bool {
    match path.extension() {
        Some(ext) => matches!(
            ext.to_string_lossy().to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp"
        ),
        None => false,
    }
}

/// Insert `item` unless an item with the same path already exists. When it does,
/// fill in dimensions/thumbnail that may be missing because the watcher picked
/// the file up while it was still being written, and keep the original id.
pub fn upsert_image_item(state: &Arc<AppState>, item: ImageItem) -> ImageItem {
    let mut images = state.images.lock();
    if let Some(existing) = images.iter_mut().find(|img| img.path == item.path) {
        if existing.width == 0 && item.width > 0 {
            existing.width = item.width;
            existing.height = item.height;
            existing.thumbnail = item.thumbnail;
        }
        return existing.clone();
    }
    images.push(item.clone());
    item
}

/// Scan the folder once, sort by modified time ascending, keep the newest `limit` items.
pub fn scan_folder(state: &Arc<AppState>, folder: &Path, limit: usize) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_image_file(p))
        .collect();

    paths.sort_by_key(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .ok()
    });

    let auto_hang = state.settings.lock().auto_hang;
    let start = paths.len().saturating_sub(limit);
    let mut images = state.images.lock();
    for path in &paths[start..] {
        let path_str = path.to_string_lossy().to_string();
        if images.iter().any(|img| img.path == path_str) {
            continue;
        }
        let mut item = create_image_item_from_path(path.clone());
        if !auto_hang {
            item.source = ImageSource::Basket;
        }
        images.push(item);
    }
}

pub fn start_file_watcher(state: Arc<AppState>, folder: PathBuf) {
    {
        let mut active = state.file_watcher_active.lock();
        if *active {
            return;
        }
        *active = true;
    }

    scan_folder(&state, &folder, 30);

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher: RecommendedWatcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("file watcher init failed: {e}");
                return;
            }
        };

        if watcher.watch(&folder, RecursiveMode::NonRecursive).is_err() {
            eprintln!("file watcher failed to watch {:?}", folder);
            return;
        }

        loop {
            if !*state.file_watcher_active.lock() {
                break;
            }
            let Ok(Ok(event)) = rx.recv_timeout(std::time::Duration::from_millis(500)) else {
                continue;
            };
            if !matches!(event.kind, EventKind::Create(_)) {
                continue;
            }
            for path in event.paths {
                if !path.is_file() || !is_image_file(&path) {
                    continue;
                }
                let path_str = path.to_string_lossy().to_string();
                if state.images.lock().iter().any(|img| img.path == path_str) {
                    continue;
                }
                let mut item = create_image_item_from_path(path.clone());
                if item.width == 0 {
                    // 应用自身保存的图片可能仍在写入，稍后重试一次
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    item = create_image_item_from_path(path);
                }
                if item.width == 0 {
                    continue;
                }
                if !state.settings.lock().auto_hang {
                    item.source = ImageSource::Basket;
                }
                let item = upsert_image_item(&state, item);
                if let Some(handle) = state.app_handle.lock().as_ref() {
                    let _ = handle.emit("image-collected", item);
                }
            }
        }
    });
}
