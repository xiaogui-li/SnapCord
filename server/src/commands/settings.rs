use tauri::{command, Emitter};
use std::sync::Arc;
use crate::store::AppState;
use crate::models::Settings;

#[command]
pub fn get_settings(state: tauri::State<Arc<AppState>>) -> Settings {
    state.settings.lock().clone()
}

#[command]
pub fn update_settings(
    app: tauri::AppHandle,
    state: tauri::State<Arc<AppState>>,
    settings: Settings,
) -> Result<(), String> {
    let previous_startup = state.settings.lock().start_on_boot;
    {
        let mut current = state.settings.lock();
        *current = settings.clone();
    }
    crate::store::save_settings(&settings);

    if previous_startup != settings.start_on_boot {
        set_registry_startup(settings.start_on_boot)?;
    }

    apply_watchers(&app, state.inner());

    if let Err(err) = crate::services::hotkey::register_hotkeys(&app, state.inner()) {
        eprintln!("{err}");
    }

    // 设置窗口是独立 webview，主置物架靠这个事件同步缩放/置顶/主题。
    let _ = app.emit("settings-updated", settings);

    Ok(())
}

#[command]
pub fn toggle_collect_images(
    app: tauri::AppHandle,
    state: tauri::State<Arc<AppState>>,
) -> Result<bool, String> {
    let enabled = {
        let mut settings = state.settings.lock();
        settings.collect_images = !settings.collect_images;
        crate::store::save_settings(&settings);
        settings.collect_images
    };
    apply_watchers(&app, state.inner());
    Ok(enabled)
}

#[command]
pub fn toggle_collect_texts(
    app: tauri::AppHandle,
    state: tauri::State<Arc<AppState>>,
) -> Result<bool, String> {
    let enabled = {
        let mut settings = state.settings.lock();
        settings.collect_texts = !settings.collect_texts;
        crate::store::save_settings(&settings);
        settings.collect_texts
    };
    apply_watchers(&app, state.inner());
    Ok(enabled)
}

#[command]
pub fn toggle_auto_hang(state: tauri::State<Arc<AppState>>) -> Result<bool, String> {
    let mut settings = state.settings.lock();
    settings.auto_hang = !settings.auto_hang;
    crate::store::save_settings(&settings);
    Ok(settings.auto_hang)
}

#[command]
pub fn set_startup(state: tauri::State<Arc<AppState>>, enable: bool) -> Result<(), String> {
    set_registry_startup(enable)?;
    let mut settings = state.settings.lock();
    settings.start_on_boot = enable;
    crate::store::save_settings(&settings);
    Ok(())
}

fn apply_watchers(_app: &tauri::AppHandle, state: &Arc<AppState>) {
    let (collect_images, collect_texts, folder) = {
        let settings = state.settings.lock();
        (
            settings.collect_images,
            settings.collect_texts,
            settings.screenshots_folder.clone(),
        )
    };

    if collect_texts {
        crate::services::clipboard_watcher::start_clipboard_watcher(Arc::clone(state));
    } else {
        crate::services::clipboard_watcher::stop_clipboard_watcher(Arc::clone(state));
    }

    if collect_images {
        if !folder.is_empty() {
            crate::services::file_watcher::start_file_watcher(
                Arc::clone(state),
                std::path::PathBuf::from(folder),
            );
        }
        crate::services::clipboard_image_watcher::start_clipboard_image_watcher(Arc::clone(state));
    } else {
        *state.file_watcher_active.lock() = false;
        *state.clipboard_image_watcher_active.lock() = false;
    }
}

fn set_registry_startup(enable: bool) -> Result<(), String> {
    use windows::core::w;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_SET_VALUE, REG_SZ,
    };

    unsafe {
        let mut key = HKEY::default();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            0,
            KEY_SET_VALUE,
            &mut key,
        )
        .ok()
        .map_err(|e| format!("打开启动项注册表失败: {e}"))?;

        let result = if enable {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let value = format!("\"{}\" --minimized", exe.display());
            let data: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
            RegSetValueExW(
                key,
                w!("SnapCord"),
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    data.as_ptr() as *const u8,
                    data.len() * 2,
                )),
            )
            .ok()
            .map_err(|e| format!("写入启动项失败: {e}"))
        } else {
            RegDeleteValueW(key, w!("SnapCord"))
                .ok()
                .map_err(|e| format!("删除启动项失败: {e}"))
        };

        let _ = RegCloseKey(key);
        result
    }
}
