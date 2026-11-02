# SnapCord 模块架构

## 后端模块 (Rust / server/src/)

```
server/src/
├── main.rs              # 入口
├── lib.rs               # Tauri 应用构建
├── commands/            # Tauri IPC 命令
│   ├── mod.rs
│   ├── clipboard.rs     # 剪贴板操作
│   ├── capture.rs       # 截图操作
│   ├── drag.rs          # 拖拽操作
│   ├── pin.rs           # 桌面贴图
│   ├── settings.rs      # 设置操作
│   └── update.rs        # 自动更新
├── services/            # 后台服务
│   ├── mod.rs
│   ├── clipboard_watcher.rs   # 剪贴板监控
│   ├── file_watcher.rs        # 截图文件夹监听
│   ├── hotkey.rs              # 全局热键
│   └── tray.rs                # 系统托盘
├── models/              # 数据模型
│   ├── mod.rs
│   ├── image.rs         # 图片模型
│   ├── text.rs          # 文字模型
│   └── settings.rs      # 设置模型
├── store.rs             # 状态存储 (AppState)
└── native/              # 原生 Windows API
    ├── mod.rs
    ├── drag_drop.rs     # COM 拖拽
    ├── layered_window.rs # 分层窗口
    └── screenshot.rs    # 屏幕捕获
```

## 前端模块 (Vue / ui/src/)

```
ui/src/
├── main.ts
├── App.vue
├── components/
│   ├── Rope.vue             # 顶部绳索栏
│   ├── ImageCard.vue        # 图片卡片
│   ├── BasketPanel.vue      # 图片篮子
│   ├── TextEnvelope.vue     # 文字信封
│   ├── PinnedWindow.vue     # 桌面贴图窗口
│   ├── CaptureOverlay.vue   # 区域截图覆盖层
│   ├── ImageEditor.vue      # 图片编辑器
│   ├── SettingsWindow.vue   # 设置窗口
│   └── Toast.vue            # 提示组件
├── composables/
│   ├── useClipboard.ts      # 剪贴板逻辑
│   ├── useDrag.ts           # 拖拽逻辑
│   ├── useCapture.ts        # 截图逻辑
│   └── useMonitor.ts        # 多显示器
├── stores/
│   ├── imageStore.ts        # 图片状态
│   ├── textStore.ts         # 文字状态
│   └── settingsStore.ts     # 设置状态
├── types/
│   └── index.ts             # 共享类型
└── assets/
    ├── main.css
    └── design-system.md
```

## IPC 接口 (Tauri Commands)

### 剪贴板
- `start_clipboard_watcher()` - 启动剪贴板监控
- `stop_clipboard_watcher()` - 停止监控
- `get_clipboard_image()` - 获取剪贴板图片
- `get_clipboard_text()` - 获取剪贴板文字
- `copy_image_to_clipboard(path: String)` - 复制图片到剪贴板
- `copy_text_to_clipboard(text: String)` - 复制文字到剪贴板

### 文件
- `start_file_watcher(folder: String)` - 启动文件监听
- `stop_file_watcher()` - 停止监听
- `save_image_copy(path: String)` - 保存图片副本
- `delete_image(path: String)` - 删除图片

### 拖拽
- `start_drag(file_path: String)` - 开始拖拽
- `create_drag_preview(image_data: Vec<u8>, x: i32, y: i32)` - 创建拖拽预览
- `update_drag_preview(hwnd: u32, x: i32, y: i32)` - 更新预览位置
- `destroy_drag_preview(hwnd: u32)` - 销毁预览

### 桌面贴图
- `create_pinned_window(image_path: String, x: i32, y: i32)` - 创建贴图窗口
- `update_pinned_window(hwnd: u32, x: i32, y: i32, scale: f64)` - 更新贴图
- `destroy_pinned_window(hwnd: u32)` - 销毁贴图

### 截图
- `start_region_capture()` - 开始区域截图
- `complete_capture(region: Rect)` - 完成截图
- `cancel_capture()` - 取消截图

### 设置
- `get_settings()` - 获取设置
- `save_settings(settings: Settings)` - 保存设置
- `get_screenshots_folder()` - 获取截图文件夹路径

### 系统
- `register_hotkey(modifiers: u32, key: u32)` - 注册热键
- `unregister_hotkey(id: u32)` - 注销热键
- `create_tray_menu()` - 创建托盘菜单
- `check_updates()` - 检查更新

## 状态模型

### Image
```typescript
interface Image {
  id: string;
  path: string;
  thumbnail: string;
  width: number;
  height: number;
  timestamp: number;
  source: 'screenshot' | 'clipboard' | 'manual';
  isPinned: boolean;
  pinHwnd?: number;
}
```

### TextItem
```typescript
interface TextItem {
  id: string;
  content: string;
  timestamp: number;
}
```

### Settings
```typescript
interface Settings {
  screenshotsFolder: string;
  autoHang: boolean;
  collectClipboardImage: boolean;
  collectClipboardText: boolean;
  captureHotkey: { modifiers: number; key: number };
  toggleHotkey: { modifiers: number; key: number };
  displayScreen: number;
  theme: string;
  startOnBoot: boolean;
  autoUpdate: boolean;
}
```

## 数据流

```
[剪贴板变化] → clipboard_watcher → [新图片/文字] → store
                                          ↓
[文件变化] → file_watcher → [新截图] → store
                                          ↓
                                    Rope 栏显示
                                    ↓
                    ┌───────────────┼───────────────┐
                    ↓               ↓               ↓
              单击复制        拖拽到外部      钉在桌面
              (clipboard)    (COM drag)    (layered window)
```
