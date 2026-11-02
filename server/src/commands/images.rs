use tauri::command;
use std::sync::Arc;
use crate::store::AppState;
use crate::models::{ImageItem, ImageSource};

#[command]
pub fn get_images(state: tauri::State<Arc<AppState>>) -> Vec<ImageItem> {
    state.images.lock().clone()
}

#[command]
pub fn add_image(state: tauri::State<Arc<AppState>>, path: String) -> Result<ImageItem, String> {
    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("File not found".to_string());
    }

    let image = crate::services::file_watcher::create_image_item_from_path(path_buf);
    state.images.lock().push(image.clone());
    Ok(image)
}

#[command]
pub fn remove_image(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let mut images = state.images.lock();
    if let Some(pos) = images.iter().position(|img| img.id == id) {
        images.remove(pos);
        Ok(())
    } else {
        Err("Image not found".to_string())
    }
}

#[command]
pub fn move_to_basket(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let mut images = state.images.lock();
    if let Some(img) = images.iter_mut().find(|img| img.id == id) {
        img.source = ImageSource::Basket;
        Ok(())
    } else {
        Err("Image not found".to_string())
    }
}

#[command]
pub fn move_to_rope(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let mut images = state.images.lock();
    if let Some(img) = images.iter_mut().find(|img| img.id == id) {
        img.source = ImageSource::Screenshot;
        Ok(())
    } else {
        Err("Image not found".to_string())
    }
}

#[command]
pub fn pin_image(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let mut images = state.images.lock();
    if let Some(img) = images.iter_mut().find(|img| img.id == id) {
        img.is_pinned = true;
        Ok(())
    } else {
        Err("Image not found".to_string())
    }
}

#[command]
pub fn unpin_image(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let mut images = state.images.lock();
    if let Some(img) = images.iter_mut().find(|img| img.id == id) {
        img.is_pinned = false;
        img.pin_hwnd = None;
        Ok(())
    } else {
        Err("Image not found".to_string())
    }
}

#[command]
pub fn copy_image(_state: tauri::State<Arc<AppState>>, path: String) -> Result<(), String> {
    crate::services::clipboard_service::copy_image_to_clipboard(&path)
}
