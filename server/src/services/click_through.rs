use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tauri::Manager;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use crate::store::AppState;

const POLL_INTERVAL: Duration = Duration::from_millis(16);

/// 置物架是一条铺满屏幕顶边的透明窗口，只有卡片和工具按钮该吃鼠标事件，
/// 其余区域必须让点击落到下面的桌面/应用上。
pub fn start_click_through(state: Arc<AppState>) {
    std::thread::spawn(move || {
        let mut ignoring = false;
        loop {
            std::thread::sleep(POLL_INTERVAL);

            let Some(app) = state.app_handle.lock().clone() else {
                continue;
            };
            let Some(window) = app.get_webview_window("main") else {
                continue;
            };

            // 原生拖拽期间保持接收事件，否则拖到一半窗口就“消失”了。
            if state.drag_active.load(Ordering::Relaxed) {
                if ignoring && window.set_ignore_cursor_events(false).is_ok() {
                    ignoring = false;
                }
                continue;
            }

            let (Ok(origin), Ok(scale)) = (window.inner_position(), window.scale_factor()) else {
                continue;
            };
            let mut point = POINT::default();
            if unsafe { GetCursorPos(&mut point) }.is_err() {
                continue;
            }
            let x = (point.x - origin.x) as f64 / scale;
            let y = (point.y - origin.y) as f64 / scale;
            let regions = state.hit_regions.lock();
            // 前端还没上报时保持可交互，避免界面异常时整条置物架点不动。
            if regions.is_empty() {
                continue;
            }
            let over_hot = regions
                .iter()
                .any(|r| x >= r[0] && x <= r[2] && y >= r[1] && y <= r[3]);
            drop(regions);
            let want_ignore = !over_hot;

            if want_ignore != ignoring && window.set_ignore_cursor_events(want_ignore).is_ok() {
                ignoring = want_ignore;
            }
        }
    });
}
