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
#[serde(default)]
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
    pub card_scale: f64,
    /// 绳索弧度，0–100（百分比，100 对应最大下垂）。
    pub rope_sag: f64,
    /// 卡片摇晃幅度（度），0 表示关闭动画。
    pub card_swing: f64,
    /// 绳索是否一直显示（false 则自动隐藏）。
    pub rope_always_visible: bool,
    /// 绳索自动隐藏时间（秒），0 表示一直显示。
    pub rope_auto_hide_seconds: f64,
    /// 绳索动画风格（0=弧度展开，1=拉绳抖动）。
    pub rope_animation_style: u8,
    /// 绳索动画时长（毫秒）。
    pub rope_animation_duration: u32,
    pub always_on_top: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            screenshots_folder: String::new(),
            auto_hang: true,
            collect_images: true,
            collect_texts: false,
            capture_hotkey_modifiers: 0x0001 | 0x0002, // Alt + Ctrl
            capture_hotkey_key: 0x53, // S
            toggle_hotkey_modifiers: 0x0001 | 0x0002, // Alt + Ctrl
            toggle_hotkey_key: 0x54, // T
            display_screen: 0,
            theme: "classic".to_string(),
            start_on_boot: false,
            auto_update: true,
            card_scale: 1.0,
            rope_sag: 35.0,
            card_swing: 4.0,
            rope_always_visible: false,
            rope_auto_hide_seconds: 5.0,
            rope_animation_style: 0,
            rope_animation_duration: 1500,
            always_on_top: true,
        }
    }
}
