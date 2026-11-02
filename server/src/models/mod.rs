use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageItem {
    pub id: String,
    pub path: String,
    pub thumbnail: String,
    pub width: u32,
    pub height: u32,
    pub timestamp: i64,
    pub source: ImageSource,
    pub is_pinned: bool,
    pub pin_hwnd: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageSource {
    #[serde(rename = "screenshot")]
    Screenshot,
    #[serde(rename = "clipboard")]
    Clipboard,
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "basket")]
    Basket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextItem {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub screenshots_folder: String,
    pub auto_hang: bool,
    pub collect_images: bool,
    pub collect_texts: bool,
    pub capture_hotkey_modifiers: u32,
    pub capture_hotkey_key: u32,
    pub toggle_hotkey_modifiers: u32,
    pub toggle_hotkey_key: u32,
    pub display_screen: i32,
    pub theme: String,
    pub start_on_boot: bool,
    pub auto_update: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            screenshots_folder: String::new(),
            auto_hang: true,
            collect_images: false,
            collect_texts: false,
            capture_hotkey_modifiers: 0x0002 | 0x0008, // Ctrl + Alt
            capture_hotkey_key: 0x53, // S
            toggle_hotkey_modifiers: 0x0002 | 0x0008, // Ctrl + Alt
            toggle_hotkey_key: 0x54, // T
            display_screen: 0,
            theme: "classic".to_string(),
            start_on_boot: false,
            auto_update: true,
        }
    }
}
