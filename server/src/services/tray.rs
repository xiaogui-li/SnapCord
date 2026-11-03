use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::store::AppState;

pub fn create_tray_menu(app: &AppHandle) -> Result<(), String> {
    let show = MenuItem::with_id(app, "show", "显示置物架", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let hide = MenuItem::with_id(app, "hide", "隐藏置物架", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let capture = MenuItem::with_id(app, "capture", "截图 (Ctrl+Alt+S)", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let basket = MenuItem::with_id(app, "basket", "图片篮子", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let texts = MenuItem::with_id(app, "texts", "文字信封", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit =
        MenuItem::with_id(app, "quit", "退出 SnapCord", true, None::<&str>).map_err(|e| e.to_string())?;
    let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let separator2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &separator,
            &capture,
            &basket,
            &texts,
            &settings,
            &separator2,
            &quit,
        ],
    )
    .map_err(|e| e.to_string())?;

    let mut builder = TrayIconBuilder::with_id("snapcord-tray")
        .tooltip("SnapCord · 图索")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "hide" => hide_main_window(app),
            "capture" => trigger_capture(app),
            "basket" => open_panel(app, "basket"),
            "texts" => open_panel(app, "texts"),
            "settings" => {
                if let Err(err) = crate::commands::system::open_settings_impl(app) {
                    eprintln!("打开设置窗口失败: {err}");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        // 被 Win+D / 最小化后窗口可能停在屏幕外，重新钉回顶边。
        let expanded = window
            .outer_size()
            .ok()
            .zip(window.scale_factor().ok())
            .map(|(size, scale)| size.height as f64 / scale > 400.0)
            .unwrap_or(false);
        crate::commands::system::apply_overlay_geometry(&window, expanded);
        let _ = window.set_focus();
        let _ = app.emit("rope-refresh", ());
    }
}

fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// 托盘菜单没有自己的界面，先把置物架叫出来，再让前端切到对应面板。
fn open_panel(app: &AppHandle, panel: &str) {
    show_main_window(app);
    let _ = app.emit("open-panel", panel);
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
            let _ = window.hide();
        } else {
            show_main_window(app);
        }
    }
}

pub fn trigger_capture(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let state = app.state::<Arc<AppState>>();
        if let Err(err) = crate::commands::capture::start_capture_impl(&app, state.inner()) {
            eprintln!("截图失败: {err}");
            let _ = app.emit("capture-error", err);
        }
    });
}
