use std::sync::Arc;

use tauri::{command, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_STYLE, WS_POPUP, WS_VISIBLE,
};

use crate::store::AppState;

/// 设置是独立窗口，可以拖到屏幕任意位置。
const SETTINGS_LABEL: &str = "settings";

#[command]
pub fn open_settings_window(app: AppHandle) -> Result<(), String> {
    open_settings_impl(&app)
}

pub fn open_settings_impl(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(
        app,
        SETTINGS_LABEL,
        WebviewUrl::App("index.html?settings=1".into()),
    )
    .title("SnapCord 设置")
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .resizable(true)
    .inner_size(440.0, 700.0)
    .min_inner_size(380.0, 480.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 把主窗口钉在主显示器左上角，铺满整个屏幕宽度，高度只覆盖绳索和卡片区域。
pub fn apply_overlay_geometry(window: &tauri::WebviewWindow, expanded: bool) {
    let app = window.app_handle();
    // setup 阶段窗口可能尚未映射到显示器上，逐级回退。
    let monitor = window
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())
        .or_else(|| {
            window
                .available_monitors()
                .ok()
                .and_then(|monitors| monitors.into_iter().next())
        });
    let Some(monitor) = monitor else {
        return;
    };
    let scale = monitor.scale_factor();
    // 窗口高度只覆盖绳索和卡片区域（屏幕高度的 40%），留出任务栏空间。
    let logical_height = (monitor.size().height as f64 / scale) * 0.4;
    let _ = window.set_size(tauri::PhysicalSize::new(
        monitor.size().width,
        (logical_height * scale).round() as u32,
    ));
    let _ = window.set_position(tauri::PhysicalPosition::new(
        monitor.position().x,
        monitor.position().y,
    ));

    // 用 WS_POPUP 完全重写窗口样式，彻底去掉所有边框和 resize 区域。
    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let raw_hwnd = windows::Win32::Foundation::HWND(hwnd.0);
            let _ = SetWindowLongPtrW(raw_hwnd, GWL_STYLE, (WS_POPUP | WS_VISIBLE).0 as isize);
        }
    }
}

#[command]
pub fn set_overlay_expanded(window: tauri::WebviewWindow, expanded: bool) -> Result<(), String> {
    apply_overlay_geometry(&window, expanded);
    Ok(())
}

/// 前端把卡片、按钮、面板的矩形（CSS 逻辑像素）上报给点击穿透线程。
#[command]
pub fn set_hit_regions(state: tauri::State<Arc<AppState>>, regions: Vec<[f64; 4]>) {
    *state.hit_regions.lock() = regions;
}

#[command]
pub fn get_screenshots_folder() -> Result<String, String> {
    // Use default Pictures/Screenshots folder
    let home = std::env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let screenshots = format!("{}\\Pictures\\Screenshots", home);
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&screenshots).map_err(|e| e.to_string())?;
    
    Ok(screenshots)
}

#[command]
pub fn open_file_in_explorer(path: String) -> Result<(), String> {
    use std::process::Command;

    Command::new("explorer")
        .arg("/select,")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open explorer: {}", e))?;



    Ok(())
}

#[command]
pub fn open_file_with_default_app(path: String) -> Result<(), String> {
    use std::process::Command;

    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open file: {}", e))?;

    Ok(())
}

#[command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
