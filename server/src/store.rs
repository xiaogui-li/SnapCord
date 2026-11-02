use std::sync::Arc;
use tauri::AppHandle;
use parking_lot::Mutex;
use crate::models::{ImageItem, TextItem, Settings};

pub struct AppState {
    pub app_handle: Mutex<Option<AppHandle>>,
    pub images: Mutex<Vec<ImageItem>>,
    pub texts: Mutex<Vec<TextItem>>,
    pub settings: Mutex<Settings>,
    pub clipboard_watcher_active: Mutex<bool>,
    pub file_watcher_active: Mutex<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            app_handle: Mutex::new(None),
            images: Mutex::new(Vec::new()),
            texts: Mutex::new(Vec::new()),
            settings: Mutex::new(Settings::default()),
            clipboard_watcher_active: Mutex::new(false),
            file_watcher_active: Mutex::new(false),
        }
    }
}
