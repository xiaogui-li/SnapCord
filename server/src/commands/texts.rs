use tauri::command;
use std::sync::Arc;
use crate::store::AppState;
use crate::models::TextItem;

#[command]
pub fn get_texts(state: tauri::State<Arc<AppState>>) -> Vec<TextItem> {
    state.texts.lock().clone()
}

#[command]
pub fn add_text(state: tauri::State<Arc<AppState>>, content: String) -> Result<TextItem, String> {
    use uuid::Uuid;
    use chrono::Utc;

    let item = TextItem {
        id: Uuid::new_v4().to_string(),
        content,
        timestamp: Utc::now().timestamp(),
    };
    state.texts.lock().push(item.clone());
    Ok(item)
}

#[command]
pub fn remove_text(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let mut texts = state.texts.lock();
    if let Some(pos) = texts.iter().position(|t| t.id == id) {
        texts.remove(pos);
        Ok(())
    } else {
        Err("Text not found".to_string())
    }
}

#[command]
pub fn clear_texts(state: tauri::State<Arc<AppState>>) -> Result<(), String> {
    state.texts.lock().clear();
    Ok(())
}

#[command]
pub fn copy_text(_state: tauri::State<Arc<AppState>>, content: String) -> Result<(), String> {
    crate::services::clipboard_watcher::copy_text_to_clipboard(content)
}
