use tauri::command;
use std::sync::Arc;
use crate::store::AppState;
use crate::models::Settings;

#[command]
pub fn get_settings(state: tauri::State<Arc<AppState>>) -> Settings {
    state.settings.lock().clone()
}

#[command]
pub fn update_settings(
    state: tauri::State<Arc<AppState>>,
    settings: Settings,
) -> Result<(), String> {
    let mut current = state.settings.lock();
    *current = settings;
    Ok(())
}

#[command]
pub fn toggle_collect_images(state: tauri::State<Arc<AppState>>) -> Result<bool, String> {
    let mut settings = state.settings.lock();
    settings.collect_images = !settings.collect_images;
    Ok(settings.collect_images)
}

#[command]
pub fn toggle_collect_texts(state: tauri::State<Arc<AppState>>) -> Result<bool, String> {
    let mut settings = state.settings.lock();
    settings.collect_texts = !settings.collect_texts;
    Ok(settings.collect_texts)
}

#[command]
pub fn toggle_auto_hang(state: tauri::State<Arc<AppState>>) -> Result<bool, String> {
    let mut settings = state.settings.lock();
    settings.auto_hang = !settings.auto_hang;
    Ok(settings.auto_hang)
}

#[command]
pub fn set_startup(_state: tauri::State<Arc<AppState>>, _enable: bool) -> Result<(), String> {
    // TODO: Implement registry-based startup with updated Windows API
    // For now, just return Ok
    Ok(())
}
