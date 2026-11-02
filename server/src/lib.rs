#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod store;
mod services;
mod commands;

use std::sync::Arc;
use store::AppState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub fn run() {
    let state = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            {
                let mut settings = state.settings.lock();
                settings.screenshots_folder = screenshots_folder.clone();
            }

            // Start file watcher
            if !screenshots_folder.is_empty() {
                services::file_watcher::start_file_watcher(
                    state.clone(),
                    std::path::PathBuf::from(screenshots_folder),
                );
            }

            // Create tray menu
            services::tray::create_tray_menu(app.handle()).ok();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
