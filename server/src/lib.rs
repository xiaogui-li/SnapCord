#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod store;
mod services;
mod commands;

use std::sync::Arc;
use tauri::Manager;
use store::AppState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub fn run() {
    let state = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Images
            commands::images::get_images,
            commands::images::add_image,
            commands::images::remove_image,
            commands::images::move_to_basket,
            commands::images::move_to_rope,
            commands::images::pin_image,
            commands::images::unpin_image,
            commands::images::copy_image,
            commands::images::open_image_editor,
            commands::images::read_image_bytes,
            commands::images::save_edited_image,
            // Drag & drop
            commands::drag::start_file_drag,
            // Capture
            commands::capture::start_capture,
            commands::capture::get_capture_frame,
            commands::capture::finish_capture,
            commands::capture::cancel_capture,
            // Texts
            commands::texts::get_texts,
            commands::texts::add_text,
            commands::texts::remove_text,
            commands::texts::clear_texts,
            commands::texts::copy_text,
            // Settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::toggle_collect_images,
            commands::settings::toggle_collect_texts,
            commands::settings::toggle_auto_hang,
            commands::settings::set_startup,
            // System
            commands::system::get_screenshots_folder,
            commands::system::open_settings_window,
            commands::system::set_overlay_expanded,
            commands::system::set_hit_regions,
            commands::system::open_file_in_explorer,
            commands::system::open_file_with_default_app,
            commands::system::get_app_version,
        ])
        .setup(move |app| {
            // Store app handle
            {
                let mut handle = state.app_handle.lock();
                *handle = Some(app.handle().clone());
            }

            // Get screenshots folder
            let screenshots_folder = commands::system::get_screenshots_folder().unwrap_or_default();
            let (collect_images, collect_texts) = {
                let mut settings = state.settings.lock();
                settings.screenshots_folder = screenshots_folder.clone();
                crate::store::save_settings(&settings);
                (settings.collect_images, settings.collect_texts)
            };

            // Start file watcher
            if collect_images && !screenshots_folder.is_empty() {
                services::file_watcher::start_file_watcher(
                    state.clone(),
                    std::path::PathBuf::from(screenshots_folder),
                );
            }

            // Clipboard screenshots only exist on the clipboard (Win+Shift+S,
            // third-party capture tools), so watch it independently of the folder.
            if collect_images {
                services::clipboard_image_watcher::start_clipboard_image_watcher(state.clone());
            }

            match app.get_webview_window("main") {
                Some(window) => commands::system::apply_overlay_geometry(&window, false),
                None => eprintln!("main window missing during setup"),
            }

            services::click_through::start_click_through(state.clone());

            if collect_texts {
                services::clipboard_watcher::start_clipboard_watcher(state.clone());
            }

            // Create tray menu
            services::tray::create_tray_menu(app.handle()).ok();

            // Register global hotkeys
            if let Err(err) = services::hotkey::register_hotkeys(app.handle(), &state) {
                eprintln!("{err}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if !matches!(event, tauri::WindowEvent::Destroyed) {
                return;
            }
            let label = window.label().to_string();
            let app = window.app_handle();
            let state = app.state::<Arc<AppState>>();

            if let Some(id) = label.strip_prefix("pin-") {
                let mut images = state.images.lock();
                if let Some(img) = images.iter_mut().find(|img| img.id == id) {
                    img.is_pinned = false;
                    img.pin_hwnd = None;
                }
            } else if let Some(seq) = label.strip_prefix("capture-") {
                let is_current = seq
                    .parse::<u64>()
                    .map(|seq| seq == *state.capture_seq.lock())
                    .unwrap_or(false);
                if !is_current {
                    return;
                }
                *state.capture_frame.lock() = None;
                let was_visible = std::mem::replace(&mut *state.main_was_visible.lock(), false);
                if was_visible {
                    services::tray::show_main_window(app);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
