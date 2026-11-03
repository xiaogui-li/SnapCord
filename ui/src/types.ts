export type ImageSource = 'screenshot' | 'clipboard' | 'manual' | 'basket';

export interface ImageItem {
  id: string;
  path: string;
  thumbnail: string;
  width: number;
  height: number;
  timestamp: number;
  source: ImageSource;
  is_pinned: boolean;
  pin_hwnd: number | null;
}

export interface TextItem {
  id: string;
  content: string;
  timestamp: number;
}

export interface Settings {
  screenshots_folder: string;
  auto_hang: boolean;
  collect_images: boolean;
  collect_texts: boolean;
  capture_hotkey_modifiers: number;
  capture_hotkey_key: number;
  toggle_hotkey_modifiers: number;
  toggle_hotkey_key: number;
  display_screen: number;
  theme: string;
  start_on_boot: boolean;
  auto_update: boolean;
  card_scale: number;
  rope_sag: number;
  card_swing: number;
  rope_always_visible: boolean;
  rope_auto_hide_seconds: number;
  rope_animation_style: number;
  rope_animation_duration: number;
  always_on_top: boolean;
}
