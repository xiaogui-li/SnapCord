use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::store::AppState;

pub fn hotkey_string(modifiers: u32, key: u32) -> String {
    let mut parts: Vec<String> = Vec::new();
    if modifiers & 0x0002 != 0 {
        parts.push("Control".to_string());
    }
    if modifiers & 0x0001 != 0 {
        parts.push("Alt".to_string());
    }
    if modifiers & 0x0004 != 0 {
        parts.push("Shift".to_string());
    }
    if modifiers & 0x0008 != 0 {
        parts.push("Super".to_string());
    }

    let name = match key {
        0x30..=0x39 | 0x41..=0x5A => char::from_u32(key)
            .map(|c| c.to_string())
            .unwrap_or_else(|| "S".to_string()),
        0x70..=0x7B => format!("F{}", key - 0x6F),
        0x20 => "Space".to_string(),
        _ => "S".to_string(),
    };
    parts.push(name);
    parts.join("+")
}

pub fn register_hotkeys(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    let settings = state.settings.lock().clone();
    let capture_hotkey = hotkey_string(
        settings.capture_hotkey_modifiers,
        settings.capture_hotkey_key,
    );
    let toggle_hotkey = hotkey_string(
        settings.toggle_hotkey_modifiers,
        settings.toggle_hotkey_key,
    );

    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    let capture: Shortcut = capture_hotkey
        .parse()
        .map_err(|e| format!("截图热键 {capture_hotkey} 无效: {e}"))?;
    manager
        .on_shortcut(capture, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                crate::services::tray::trigger_capture(app);
            }
        })
        .map_err(|e| format!("注册截图热键失败: {e}"))?;

    let toggle: Shortcut = toggle_hotkey
        .parse()
        .map_err(|e| format!("显示热键 {toggle_hotkey} 无效: {e}"))?;
    manager
        .on_shortcut(toggle, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                crate::services::tray::show_main_window(app);
            }
        })
        .map_err(|e| format!("注册显示热键失败: {e}"))?;

    Ok(())
}
