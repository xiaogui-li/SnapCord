use tauri::{command, Emitter, Manager};
use std::sync::Arc;
use base64::{engine::general_purpose::STANDARD, Engine as _};
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
        return Err("文件不存在".to_string());
    }

    {
        let images = state.images.lock();
        if let Some(existing) = images.iter().find(|img| img.path == path) {
            return Ok(existing.clone());
        }
    }

    let mut image = crate::services::file_watcher::create_image_item_from_path(path_buf);
    image.source = ImageSource::Manual;
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
        Err("图片不存在".to_string())
    }
}

#[command]
pub fn move_to_basket(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let mut images = state.images.lock();
    if let Some(img) = images.iter_mut().find(|img| img.id == id) {
        img.source = ImageSource::Basket;
        Ok(())
    } else {
        Err("图片不存在".to_string())
    }
}

#[command]
pub fn move_to_rope(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let mut images = state.images.lock();
    if let Some(img) = images.iter_mut().find(|img| img.id == id) {
        img.source = ImageSource::Screenshot;
        Ok(())
    } else {
        Err("图片不存在".to_string())
    }
}

#[command]
pub async fn pin_image(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    let (path, width, height) = {
        let mut images = state.images.lock();
        let img = images
            .iter_mut()
            .find(|img| img.id == id)
            .ok_or("图片不存在")?;
        if img.is_pinned {
            return Ok(());
        }
        img.is_pinned = true;
        (img.path.clone(), img.width, img.height)
    };

    let label = format!("pin-{id}");
    let encoded = percent_encode(&path);
    let url = format!("index.html?pin={encoded}&id={id}");

    let mut builder = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::App(url.into()),
    )
    .title("SnapCord")
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .resizable(true)
    .shadow(false);

    if width > 0 && height > 0 {
        let scale = (480.0 / width as f64).min(1.0);
        builder = builder.inner_size(
            (width as f64 * scale).max(120.0),
            (height as f64 * scale).max(80.0),
        );
    } else {
        builder = builder.inner_size(400.0, 300.0);
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    if let Ok(hwnd) = window.hwnd() {
        let mut images = state.images.lock();
        if let Some(img) = images.iter_mut().find(|img| img.id == id) {
            img.pin_hwnd = Some(hwnd.0 as u32);
        }
    }

    Ok(())
}

#[command]
pub async fn unpin_image(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&format!("pin-{id}")) {
        let _ = window.close();
    }

    let mut images = state.images.lock();
    if let Some(img) = images.iter_mut().find(|img| img.id == id) {
        img.is_pinned = false;
        img.pin_hwnd = None;
    }
    Ok(())
}

#[command]
pub fn copy_image(_state: tauri::State<Arc<AppState>>, path: String) -> Result<(), String> {
    crate::services::clipboard_service::copy_image_to_clipboard(&path)
}

#[command]
pub async fn open_image_editor(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    let (width, height) = {
        let images = state.images.lock();
        let img = images.iter().find(|img| img.id == id).ok_or("图片不存在")?;
        (img.width, img.height)
    };

    let label = format!("editor-{id}");
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let scale = (1100.0 / width.max(1) as f64)
        .min(820.0 / height.max(1) as f64)
        .min(1.0);
    let window_width = (width.max(1) as f64 * scale).clamp(560.0, 1400.0);
    let window_height = (height.max(1) as f64 * scale + 108.0).clamp(420.0, 980.0);

    tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::App(format!("index.html?editor=1&id={id}").into()),
    )
    .title("SnapCord 编辑")
    .decorations(false)
    .transparent(true)
    .resizable(true)
    .inner_size(window_width, window_height)
    .min_inner_size(560.0, 420.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn read_image_bytes(path: String) -> Result<tauri::ipc::Response, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[command]
pub async fn save_edited_image(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    data_url: String,
) -> Result<ImageItem, String> {
    let encoded = data_url
        .split_once(',')
        .map(|(_, data)| data)
        .unwrap_or(&data_url);
    let bytes = STANDARD
        .decode(encoded.trim())
        .map_err(|e| format!("解析编辑图片失败: {e}"))?;
    let image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;

    let folder = {
        let settings = state.settings.lock();
        let folder = settings.screenshots_folder.clone();
        if folder.is_empty() {
            std::env::temp_dir().join("SnapCord").to_string_lossy().to_string()
        } else {
            folder
        }
    };
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;

    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%3f");
    let file_path = std::path::PathBuf::from(&folder).join(format!("SnapCord_edit_{stamp}.png"));
    image.save(&file_path).map_err(|e| e.to_string())?;

    let mut item = crate::services::file_watcher::create_image_item_from_path(file_path);
    item.source = if state.settings.lock().auto_hang {
        ImageSource::Screenshot
    } else {
        ImageSource::Basket
    };
    let item = crate::services::file_watcher::upsert_image_item(&state, item);
    let _ = app.emit("image-collected", &item);

    Ok(item)
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
