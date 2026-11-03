import { invoke } from '@tauri-apps/api/core';
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ImageItem, TextItem, Settings } from './types';

export const api = {
  getImages: () => invoke<ImageItem[]>('get_images'),
  addImage: (path: string) => invoke<ImageItem>('add_image', { path }),
  removeImage: (id: string) => invoke<void>('remove_image', { id }),
  moveToBasket: (id: string) => invoke<void>('move_to_basket', { id }),
  moveToRope: (id: string) => invoke<void>('move_to_rope', { id }),
  pinImage: (id: string) => invoke<void>('pin_image', { id }),
  unpinImage: (id: string) => invoke<void>('unpin_image', { id }),
  copyImage: (path: string) => invoke<void>('copy_image', { path }),
  startFileDrag: (id: string) => invoke<void>('start_file_drag', { id }),
  openImageEditor: (id: string) => invoke<void>('open_image_editor', { id }),
  readImageBytes: (path: string) => invoke<ArrayBuffer>('read_image_bytes', { path }),
  saveEditedImage: (dataUrl: string) => invoke<ImageItem>('save_edited_image', { dataUrl }),

  startCapture: () => invoke<void>('start_capture'),
  getCaptureFrame: () => invoke<string>('get_capture_frame'),
  finishCapture: (rect: { x: number; y: number; width: number; height: number; overlay?: string }) =>
    invoke<ImageItem>('finish_capture', rect),
  cancelCapture: () => invoke<void>('cancel_capture'),

  getTexts: () => invoke<TextItem[]>('get_texts'),
  addText: (content: string) => invoke<TextItem>('add_text', { content }),
  removeText: (id: string) => invoke<void>('remove_text', { id }),
  clearTexts: () => invoke<void>('clear_texts'),
  copyText: (content: string) => invoke<void>('copy_text', { content }),

  getSettings: () => invoke<Settings>('get_settings'),
  updateSettings: (settings: Settings) => invoke<void>('update_settings', { settings }),
  toggleCollectImages: () => invoke<boolean>('toggle_collect_images'),
  toggleCollectTexts: () => invoke<boolean>('toggle_collect_texts'),
  toggleAutoHang: () => invoke<boolean>('toggle_auto_hang'),
  setStartup: (enable: boolean) => invoke<void>('set_startup', { enable }),

  setOverlayExpanded: (expanded: boolean) => invoke<void>('set_overlay_expanded', { expanded }),
  setHitRegions: (regions: number[][]) => invoke<void>('set_hit_regions', { regions }),
  getScreenshotsFolder: () => invoke<string>('get_screenshots_folder'),
  openInExplorer: (path: string) => invoke<void>('open_file_in_explorer', { path }),
  openWithDefaultApp: (path: string) => invoke<void>('open_file_with_default_app', { path }),
  getAppVersion: () => invoke<string>('get_app_version'),
};

export const events = {
  onImageCollected: (cb: (image: ImageItem) => void): Promise<UnlistenFn> =>
    listen<ImageItem>('image-collected', (e) => cb(e.payload)),
  onTextCollected: (cb: (item: TextItem) => void): Promise<UnlistenFn> =>
    listen<TextItem>('text-collected', (e) => cb(e.payload)),
  onOpenPanel: (cb: (panel: 'basket' | 'texts') => void): Promise<UnlistenFn> =>
    listen<'basket' | 'texts'>('open-panel', (e) => cb(e.payload)),
  onSettingsUpdated: (cb: (settings: Settings) => void): Promise<UnlistenFn> =>
    listen<Settings>('settings-updated', (e) => cb(e.payload)),
  // 拖动滑块时只广播预览，松手才写盘。
  emitCardScalePreview: (scale: number): Promise<void> => emit('card-scale-preview', scale),
  onCardScalePreview: (cb: (scale: number) => void): Promise<UnlistenFn> =>
    listen<number>('card-scale-preview', (e) => cb(e.payload)),
  emitRopeSagPreview: (percent: number): Promise<void> => emit('rope-sag-preview', percent),
  onRopeSagPreview: (cb: (percent: number) => void): Promise<UnlistenFn> =>
    listen<number>('rope-sag-preview', (e) => cb(e.payload)),
  emitCardSwingPreview: (degrees: number): Promise<void> => emit('card-swing-preview', degrees),
  onCardSwingPreview: (cb: (degrees: number) => void): Promise<UnlistenFn> =>
    listen<number>('card-swing-preview', (e) => cb(e.payload)),
};

export const isTauri = () => '__TAURI_INTERNALS__' in window;
