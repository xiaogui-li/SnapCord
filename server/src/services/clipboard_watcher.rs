use std::sync::Arc;
use tauri::Emitter;
use crate::store::AppState;
use crate::models::TextItem;
use uuid::Uuid;

pub fn start_clipboard_watcher(state: Arc<AppState>) {
    let mut active = state.clipboard_watcher_active.lock();
    if *active {
        return;
    }
    *active = true;
    drop(active);

    let state_clone = Arc::clone(&state);
    std::thread::spawn(move || {
        let mut last_text = String::new();

        loop {
            {
                let active = state_clone.clipboard_watcher_active.lock();
                if !*active {
                    break;
                }
            }

            // Poll clipboard for text changes
            if let Ok(text) = get_clipboard_text() {
                if !text.is_empty() && text != last_text {
                    last_text = text.clone();
                    
                    if let Some(handle) = state_clone.app_handle.lock().as_ref() {
                        let item = TextItem {
                            id: Uuid::new_v4().to_string(),
                            content: text,
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        state_clone.texts.lock().push(item.clone());
                        let _ = handle.emit("text-collected", item);
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

pub fn stop_clipboard_watcher(state: Arc<AppState>) {
    let mut active = state.clipboard_watcher_active.lock();
    *active = false;
}

fn get_clipboard_text() -> Result<String, String> {
    use clipboard_win::formats::Unicode;
    use clipboard_win::get_clipboard;

    get_clipboard::<String, _>(Unicode).map_err(|e| e.to_string())
}

pub fn get_clipboard_text_command() -> Result<String, String> {
    get_clipboard_text()
}

pub fn copy_text_to_clipboard(text: String) -> Result<(), String> {
    use clipboard_win::formats::Unicode;
    use clipboard_win::set_clipboard;

    set_clipboard(Unicode, &text).map_err(|e| e.to_string())
}
