use std::slice;
use std::sync::Arc;
use std::time::Duration;

use image::{Rgba, RgbaImage};
use tauri::{Emitter, Manager};
use windows::core::w;
use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, GetClipboardSequenceNumber, IsClipboardFormatAvailable,
    RegisterClipboardFormatW,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

use crate::models::ImageSource;
use crate::services::clipboard_service::{open_clipboard_with_retry, own_copy_sequence};
use crate::services::file_watcher::create_image_item_from_path;
use crate::store::AppState;

const CF_DIB: u32 = 8;
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// 监听剪贴板图片（Win+Shift+S、QQ/微信截图等只写剪贴板的场景）。
///
/// 用剪贴板序列号判断变化，避免每次轮询都解析图片；应用自己复制图片时
/// 记下序列号并跳过，防止把刚拖出去的图又挂回来一份。
pub fn start_clipboard_image_watcher(state: Arc<AppState>) {
    {
        let mut active = state.clipboard_image_watcher_active.lock();
        if *active {
            return;
        }
        *active = true;
    }

    std::thread::spawn(move || {
        let mut last_sequence = unsafe { GetClipboardSequenceNumber() };

        loop {
            if !*state.clipboard_image_watcher_active.lock() {
                break;
            }
            std::thread::sleep(POLL_INTERVAL);

            let sequence = unsafe { GetClipboardSequenceNumber() };
            if sequence == last_sequence {
                continue;
            }
            last_sequence = sequence;
            if sequence == own_copy_sequence() {
                continue;
            }

            let has_png = unsafe { IsClipboardFormatAvailable(png_format()) }.is_ok();
            let has_dib = unsafe { IsClipboardFormatAvailable(CF_DIB) }.is_ok();
            if !has_png && !has_dib {
                continue;
            }

            let Some(image) = read_clipboard_image(has_png) else {
                continue;
            };
            let Some(handle) = state.app_handle.lock().clone() else {
                continue;
            };
            let Some(path) = save_clipboard_image(&handle, &image) else {
                continue;
            };

            let mut item = create_image_item_from_path(path);
            if item.width == 0 {
                continue;
            }
            let auto_hang = state.settings.lock().auto_hang;
            item.source = if auto_hang {
                ImageSource::Clipboard
            } else {
                ImageSource::Basket
            };

            state.images.lock().push(item.clone());
            let _ = handle.emit("image-collected", item);
        }
    });
}

fn png_format() -> u32 {
    unsafe { RegisterClipboardFormatW(w!("PNG")) }
}

fn read_clipboard_image(prefer_png: bool) -> Option<RgbaImage> {
    unsafe {
        open_clipboard_with_retry().ok()?;
        let image = if prefer_png {
            read_png_format().or_else(|| read_dib_format())
        } else {
            read_dib_format().or_else(|| read_png_format())
        };
        let _ = CloseClipboard();
        image
    }
}

unsafe fn read_png_format() -> Option<RgbaImage> {
    let format = png_format();
    if format == 0 || IsClipboardFormatAvailable(format).is_err() {
        return None;
    }
    let handle = GetClipboardData(format).ok()?;
    let memory = HGLOBAL(handle.0);
    let pointer = GlobalLock(memory) as *const u8;
    if pointer.is_null() {
        return None;
    }
    let data = slice::from_raw_parts(pointer, GlobalSize(memory));
    let image = image::load_from_memory(data).ok().map(|img| img.to_rgba8());
    let _ = GlobalUnlock(memory);
    image
}

unsafe fn read_dib_format() -> Option<RgbaImage> {
    let handle: HANDLE = GetClipboardData(CF_DIB).ok()?;
    let memory = HGLOBAL(handle.0);
    let pointer = GlobalLock(memory) as *const u8;
    if pointer.is_null() {
        return None;
    }
    let data = slice::from_raw_parts(pointer, GlobalSize(memory));
    let image = decode_dib(data);
    let _ = GlobalUnlock(memory);
    image
}

fn decode_dib(data: &[u8]) -> Option<RgbaImage> {
    if data.len() < 40 {
        return None;
    }

    let read_u32 = |offset: usize| -> Option<u32> {
        let bytes = data.get(offset..offset + 4)?;
        Some(u32::from_le_bytes(bytes.try_into().ok()?))
    };

    let header_size = read_u32(0)? as usize;
    let width = read_u32(4)? as i32;
    let raw_height = read_u32(8)? as i32;
    let bit_count = u16::from_le_bytes(data.get(14..16)?.try_into().ok()?) as usize;
    let compression = read_u32(16)?;
    if header_size < 40 || width <= 0 || raw_height == 0 {
        return None;
    }
    if !matches!(compression, 0 | 3 | 6) {
        return None;
    }

    let top_down = raw_height < 0;
    let height = raw_height.unsigned_abs() as usize;
    let width = width as usize;
    let bytes_per_pixel = match bit_count {
        24 => 3,
        32 => 4,
        _ => return None,
    };

    // BI_BITFIELDS / BI_ALPHABITFIELDS 的通道掩码跟在 40 字节头之后，
    // 只接受常规 BGRA 排布，其余格式交给系统截图流程处理。
    let mut pixel_offset = header_size;
    if compression == 3 || compression == 6 {
        let mask_bytes = if compression == 3 { 12 } else { 16 };
        let red = read_u32(pixel_offset)?;
        let green = read_u32(pixel_offset + 4)?;
        let blue = read_u32(pixel_offset + 8)?;
        if (red, green, blue) != (0x00FF_0000, 0x0000_FF00, 0x0000_00FF) {
            return None;
        }
        pixel_offset += mask_bytes;
    }

    let stride = (width * bytes_per_pixel + 3) / 4 * 4;
    if data.len() < pixel_offset + stride * height {
        return None;
    }

    let mut image = RgbaImage::new(width as u32, height as u32);
    let mut has_alpha = false;
    for y in 0..height {
        let row = if top_down { y } else { height - 1 - y };
        let line = &data[pixel_offset + row * stride..];
        for x in 0..width {
            let px = &line[x * bytes_per_pixel..];
            let alpha = if bytes_per_pixel == 4 { px[3] } else { 255 };
            has_alpha |= alpha != 0;
            image.put_pixel(x as u32, y as u32, Rgba([px[2], px[1], px[0], alpha]));
        }
    }

    // 32bpp BI_RGB 的 alpha 通道通常全 0 且无意义，视为不透明。
    if !has_alpha {
        for pixel in image.pixels_mut() {
            pixel[3] = 255;
        }
    }

    Some(image)
}

fn save_clipboard_image(app: &tauri::AppHandle, image: &RgbaImage) -> Option<std::path::PathBuf> {
    let folder = app.path().app_data_dir().ok()?.join("clipboard");
    std::fs::create_dir_all(&folder).ok()?;
    let name = format!(
        "clip-{}-{}.png",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        &uuid::Uuid::new_v4().to_string()[..8]
    );
    let path = folder.join(name);
    image.save(&path).ok()?;
    Some(path)
}
