use std::path::PathBuf;
use tauri::AppHandle;
use parking_lot::Mutex;
use crate::models::{ImageItem, TextItem, Settings};
use crate::services::screen_capture::CaptureFrame;

pub struct AppState {
    pub app_handle: Mutex<Option<AppHandle>>,
    pub images: Mutex<Vec<ImageItem>>,
    pub texts: Mutex<Vec<TextItem>>,
    pub settings: Mutex<Settings>,
    pub clipboard_watcher_active: Mutex<bool>,
    pub clipboard_image_watcher_active: Mutex<bool>,
    pub file_watcher_active: Mutex<bool>,
    pub capture_frame: Mutex<Option<CaptureFrame>>,
    pub main_was_visible: Mutex<bool>,
    pub capture_seq: Mutex<u64>,
    /// 置物架窗口的透明区域需要穿透点击，这里保存前端上报的可交互矩形
    /// （CSS 逻辑像素，相对于窗口内容区）。
    pub hit_regions: Mutex<Vec<[f64; 4]>>,
    pub drag_active: std::sync::atomic::AtomicBool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            app_handle: Mutex::new(None),
            images: Mutex::new(Vec::new()),
            texts: Mutex::new(Vec::new()),
            settings: Mutex::new(load_settings().unwrap_or_default()),
            clipboard_watcher_active: Mutex::new(false),
            clipboard_image_watcher_active: Mutex::new(false),
            file_watcher_active: Mutex::new(false),
            capture_frame: Mutex::new(None),
            main_was_visible: Mutex::new(false),
            capture_seq: Mutex::new(0),
            hit_regions: Mutex::new(Vec::new()),
            drag_active: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

fn settings_file() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("SnapCord").join("settings.json")
}

fn load_settings() -> Option<Settings> {
    let data = std::fs::read_to_string(settings_file()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_settings(settings: &Settings) {
    let path = settings_file();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, data);
    }
}
