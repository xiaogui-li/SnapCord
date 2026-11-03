use std::sync::atomic::{AtomicU32, Ordering};

use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardSequenceNumber, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

const CF_DIB: u32 = 8;
const CF_HDROP: u32 = 15;
const DROPFILES_SIZE: u32 = 20;

static OWN_COPY_SEQUENCE: AtomicU32 = AtomicU32::new(0);

/// 应用自己写剪贴板后记录的序列号，剪贴板监听据此跳过自己的复制。
pub fn own_copy_sequence() -> u32 {
    OWN_COPY_SEQUENCE.load(Ordering::Relaxed)
}

pub fn copy_image_to_clipboard(path: &str) -> Result<(), String> {
    let source = std::path::Path::new(path);
    if !source.is_file() {
        return Err("文件不存在".to_string());
    }

    let img = image::open(source).map_err(|e| format!("无法读取图片: {e}"))?;
    let dib = build_dib(&img);
    let drop = build_dropfiles(source);

    unsafe {
        open_clipboard_with_retry()?;

        if let Err(e) = EmptyClipboard() {
            CloseClipboard().ok();
            return Err(format!("清空剪贴板失败: {e}"));
        }

        // CF_DIB: 支持直接粘贴为图片
        match alloc_global(&dib) {
            Ok(handle) => {
                if let Err(e) = SetClipboardData(CF_DIB, HANDLE(handle.0)) {
                    CloseClipboard().ok();
                    return Err(format!("写入图片数据失败: {e}"));
                }
            }
            Err(e) => {
                CloseClipboard().ok();
                return Err(e);
            }
        }

        // CF_HDROP: 支持粘贴为文件（聊天软件、资源管理器）
        match alloc_global(&drop) {
            Ok(handle) => {
                if let Err(e) = SetClipboardData(CF_HDROP, HANDLE(handle.0)) {
                    CloseClipboard().ok();
                    return Err(format!("写入文件路径失败: {e}"));
                }
            }
            Err(e) => {
                CloseClipboard().ok();
                return Err(e);
            }
        }

        CloseClipboard().map_err(|e| format!("关闭剪贴板失败: {e}"))?;
        OWN_COPY_SEQUENCE.store(GetClipboardSequenceNumber(), Ordering::Relaxed);
    }

    Ok(())
}

pub unsafe fn open_clipboard_with_retry() -> Result<(), String> {
    for _ in 0..10 {
        if OpenClipboard(HWND::default()).is_ok() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Err("剪贴板被其他程序占用".to_string())
}

unsafe fn alloc_global(data: &[u8]) -> Result<HGLOBAL, String> {
    let handle = GlobalAlloc(GMEM_MOVEABLE, data.len()).map_err(|e| e.to_string())?;
    let ptr = GlobalLock(handle);
    if ptr.is_null() {
        return Err("剪贴板内存分配失败".to_string());
    }
    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
    GlobalUnlock(handle).ok();
    Ok(handle)
}

fn build_dropfiles(path: &std::path::Path) -> Vec<u8> {
    let wide: Vec<u16> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .chain(std::iter::once(0))
        .collect();

    let mut out = Vec::with_capacity(DROPFILES_SIZE as usize + wide.len() * 2);
    out.extend_from_slice(&DROPFILES_SIZE.to_le_bytes()); // pFiles offset
    out.extend_from_slice(&0i32.to_le_bytes()); // pt.x
    out.extend_from_slice(&0i32.to_le_bytes()); // pt.y
    out.extend_from_slice(&0i32.to_le_bytes()); // fNC
    out.extend_from_slice(&1i32.to_le_bytes()); // fWide
    for unit in wide {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

/// 32bpp bottom-up DIB, alpha composited over white so transparent PNGs look correct.
fn build_dib(img: &image::DynamicImage) -> Vec<u8> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixel_bytes = width as u32 * height as u32 * 4;

    let mut out = Vec::with_capacity(40 + pixel_bytes as usize);
    out.extend_from_slice(&40u32.to_le_bytes()); // biSize
    out.extend_from_slice(&(width as i32).to_le_bytes()); // biWidth
    out.extend_from_slice(&(height as i32).to_le_bytes()); // biHeight
    out.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    out.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    out.extend_from_slice(&0u32.to_le_bytes()); // biCompression = BI_RGB
    out.extend_from_slice(&pixel_bytes.to_le_bytes()); // biSizeImage
    out.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    out.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    out.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    out.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    for y in (0..height).rev() {
        for x in 0..width {
            let px = rgba.get_pixel(x, y);
            let alpha = px[3] as u32;
            let inv = 255 - alpha;
            let r = (px[0] as u32 * alpha + 255 * inv) / 255;
            let g = (px[1] as u32 * alpha + 255 * inv) / 255;
            let b = (px[2] as u32 * alpha + 255 * inv) / 255;
            out.extend_from_slice(&[b as u8, g as u8, r as u8, 255]);
        }
    }

    out
}
