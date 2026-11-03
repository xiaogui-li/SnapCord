use std::sync::Arc;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri::{command, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::models::{ImageItem, ImageSource};
use crate::services::screen_capture;
use crate::store::AppState;

const CAPTURE_LABEL_PREFIX: &str = "capture-";

fn capture_label(state: &AppState) -> String {
    format!("{CAPTURE_LABEL_PREFIX}{}", *state.capture_seq.lock())
}

fn close_capture_window(app: &tauri::AppHandle, state: &AppState) {
    if let Some(window) = app.get_webview_window(&capture_label(state)) {
        let _ = window.close();
    }
}

fn discard_stale_capture_windows(app: &tauri::AppHandle) {
    let labels: Vec<String> = app
        .webview_windows()
        .keys()
        .filter(|label| label.starts_with(CAPTURE_LABEL_PREFIX))
        .cloned()
        .collect();
    for label in labels {
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.destroy();
        }
    }
}

#[cfg(windows)]
fn repaint_region(x: i32, y: i32, width: i32, height: i32) {
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::Graphics::Gdi::{
        RedrawWindow, HRGN, RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW,
    };
    let rect = RECT {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };
    unsafe {
        // Hiding a transparent window can leave stale DWM content on screen,
        // which the GDI screen capture would otherwise pick up.
        let _ = RedrawWindow(
            HWND(std::ptr::null_mut()),
            Some(&rect as *const RECT),
            HRGN(std::ptr::null_mut()),
            RDW_INVALIDATE | RDW_ALLCHILDREN | RDW_UPDATENOW | RDW_ERASE,
        );
    }
}

#[cfg(not(windows))]
fn repaint_region(_x: i32, _y: i32, _width: i32, _height: i32) {}

fn restore_main_window(app: &tauri::AppHandle, state: &Arc<AppState>) {
    let was_visible = {
        let mut flag = state.main_was_visible.lock();
        std::mem::replace(&mut *flag, false)
    };
    if was_visible {
        crate::services::tray::show_main_window(app);
    }
}

pub fn start_capture_impl(app: &tauri::AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    discard_stale_capture_windows(app);
    let seq = {
        let mut seq = state.capture_seq.lock();
        *seq += 1;
        *seq
    };

    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        *state.main_was_visible.lock() = visible;
        if visible {
            let position = window.outer_position().ok();
            let size = window.outer_size().ok();
            let _ = window.hide();
            for _ in 0..20 {
                if !window.is_visible().unwrap_or(false) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            if let (Some(position), Some(size)) = (position, size) {
                repaint_region(
                    position.x,
                    position.y,
                    size.width as i32,
                    size.height as i32,
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(800));
        }
    }

    let frame = match screen_capture::capture_virtual_screen() {
        Ok(frame) => frame,
        Err(err) => {
            restore_main_window(app, state);
            return Err(err);
        }
    };
    let (origin_x, origin_y) = (frame.origin_x, frame.origin_y);
    let (width, height) = frame.image.dimensions();
    *state.capture_frame.lock() = Some(frame);

    let label = format!("{CAPTURE_LABEL_PREFIX}{seq}");
    let window = match WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::App("index.html?capture=1".into()),
    )
    .title("SnapCord Capture")
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .visible(false)
    .build()
    {
        Ok(window) => window,
        Err(err) => {
            restore_main_window(app, state);
            return Err(err.to_string());
        }
    };

    let layout = (|| -> Result<(), String> {
        window
            .set_position(tauri::PhysicalPosition::new(origin_x, origin_y))
            .map_err(|e| e.to_string())?;
        window
            .set_size(tauri::PhysicalSize::new(width, height))
            .map_err(|e| e.to_string())?;
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    })();

    if let Err(err) = layout {
        let _ = window.destroy();
        restore_main_window(app, state);
        return Err(err);
    }

    Ok(())
}

pub fn cancel_capture_impl(app: &tauri::AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    close_capture_window(app, state);
    *state.capture_frame.lock() = None;
    restore_main_window(app, state);
    Ok(())
}

pub fn finish_capture_impl(
    app: &tauri::AppHandle,
    state: &Arc<AppState>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    overlay: Option<String>,
) -> Result<ImageItem, String> {
    let frame = state
        .capture_frame
        .lock()
        .take()
        .ok_or_else(|| "没有可用的截图".to_string())?;

    let (frame_w, frame_h) = frame.image.dimensions();
    let x = x.clamp(0, frame_w as i32 - 1);
    let y = y.clamp(0, frame_h as i32 - 1);
    let width = width.min(frame_w - x as u32).max(1);
    let height = height.min(frame_h - y as u32).max(1);

    let mut cropped = image::imageops::crop_imm(&frame.image, x as u32, y as u32, width, height).to_image();

    if let Some(data_url) = overlay {
        let encoded = data_url.split_once(',').map(|(_, data)| data).unwrap_or(&data_url);
        let bytes = STANDARD
            .decode(encoded.trim())
            .map_err(|e| format!("解析标注数据失败: {e}"))?;
        let layer = image::load_from_memory(&bytes)
            .map_err(|e| format!("解析标注图片失败: {e}"))?
            .to_rgba8();
        let layer = if layer.dimensions() != cropped.dimensions() {
            image::imageops::resize(
                &layer,
                cropped.width(),
                cropped.height(),
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            layer
        };
        image::imageops::overlay(&mut cropped, &layer, 0, 0);
    }

    save_capture_image(app, state, image::DynamicImage::ImageRgba8(cropped))
}

fn save_capture_image(
    app: &tauri::AppHandle,
    state: &Arc<AppState>,
    image: image::DynamicImage,
) -> Result<ImageItem, String> {
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
    let file_path = std::path::PathBuf::from(&folder).join(format!("SnapCord_{stamp}.png"));
    image.save(&file_path).map_err(|e| e.to_string())?;

    let mut item = crate::services::file_watcher::create_image_item_from_path(file_path);
    if state.settings.lock().auto_hang {
        item.source = ImageSource::Screenshot;
    } else {
        item.source = ImageSource::Basket;
    }

    let item = crate::services::file_watcher::upsert_image_item(state, item);
    let _ = app.emit("image-collected", &item);
    close_capture_window(app, state);
    restore_main_window(app, state);
    Ok(item)
}

#[command]
pub async fn start_capture(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    start_capture_impl(&app, &state)
}

#[command]
pub async fn get_capture_frame(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    let frame = state.capture_frame.lock();
    let frame = frame.as_ref().ok_or_else(|| "没有可用的截图".to_string())?;

    let rgb = image::DynamicImage::ImageRgba8(frame.image.clone()).into_rgb8();
    let mut buffer = std::io::Cursor::new(Vec::new());
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 88);
    encoder
        .encode(&rgb, rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "data:image/jpeg;base64,{}",
        STANDARD.encode(buffer.into_inner())
    ))
}

#[command]
pub async fn finish_capture(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    overlay: Option<String>,
) -> Result<ImageItem, String> {
    finish_capture_impl(&app, &state, x, y, width, height, overlay)
}

#[command]
pub async fn cancel_capture(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    cancel_capture_impl(&app, &state)
}
