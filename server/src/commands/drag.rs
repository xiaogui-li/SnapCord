use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use base64::Engine as _;
use tauri::command;
use windows::core::w;
use windows::Win32::Foundation::{
    COLORREF, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, DIB_RGB_COLORS, AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos,
    GetMessageW, KillTimer, PostThreadMessageW, RegisterClassW, SetTimer, SetWindowPos,
    SetWindowsHookExW, ShowWindow, TranslateMessage, UnhookWindowsHookEx, UpdateLayeredWindow,
    CS_HREDRAW, CS_VREDRAW, HMENU, HTTRANSPARENT, HHOOK, MSG, MSLLHOOKSTRUCT, SWP_NOACTIVATE,
    SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_SHOWNOACTIVATE, ULW_ALPHA, WH_MOUSE_LL,
    WM_MOUSEMOVE, WM_NCHITTEST, WM_QUIT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

use crate::store::AppState;

const DRAG_THUMB_MAX: u32 = 160;
// 光标落在缩略图内部，像资源管理器拖文件时那样，小幅跟随误差不易被察觉。
const GHOST_OFFSET_X: i32 = -12;
const GHOST_OFFSET_Y: i32 = -12;
// 屏幕合成比鼠标事件慢约一帧，用速度前瞻补掉这部分视觉滞后。
const LEAD_MS: f32 = 10.0;
const MAX_LEAD: f32 = 20.0;
const SETTLE_TIMER_ID: usize = 1;
const SETTLE_MS: u32 = 45;

static GHOST_HWND: AtomicIsize = AtomicIsize::new(0);

thread_local! {
    static LAST_MOTION: RefCell<Option<(i32, i32, Instant)>> = const { RefCell::new(None) };
    static VELOCITY: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
}

/// 拖拽期间跟随鼠标的缩略图窗口。
///
/// OLE 的 IDragSourceHelper 拖拽图只在支持它的放置目标（如资源管理器）里
/// 才会被绘制，应用内部拖动时完全看不到。这里自绘一个置顶透明小窗，并用
/// WH_MOUSE_LL 钩子驱动：鼠标每移动一次就同步挪窗，避免轮询受系统计时器
/// 粒度（约 15.6ms）影响而明显滞后。
struct DragGhost {
    stop: Arc<AtomicBool>,
    thread_id: u32,
    join: Option<JoinHandle<()>>,
}

impl DragGhost {
    fn start(pixels: image::RgbaImage) -> Option<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let (ready_tx, ready_rx) = mpsc::channel();

        let join = std::thread::spawn(move || {
            let hwnd = match unsafe { create_ghost_window(&pixels) } {
                Some(hwnd) => hwnd,
                None => {
                    let _ = ready_tx.send(None);
                    return;
                }
            };
            GHOST_HWND.store(hwnd.0 as isize, Ordering::SeqCst);

            unsafe {
                match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), HINSTANCE::default(), 0) {
                    Ok(hook) => {
                        let _ = ready_tx.send(Some(GetCurrentThreadId()));
                        let mut message = MSG::default();
                        while GetMessageW(&mut message, HWND::default(), 0, 0).0 > 0 {
                            let _ = TranslateMessage(&message);
                            DispatchMessageW(&message);
                        }
                        let _ = UnhookWindowsHookEx(hook);
                    }
                    Err(_) => {
                        let _ = ready_tx.send(Some(GetCurrentThreadId()));
                        while !thread_stop.load(Ordering::Relaxed) {
                            move_to_cursor(hwnd);
                            std::thread::sleep(Duration::from_millis(4));
                        }
                    }
                }
            }

            GHOST_HWND.store(0, Ordering::SeqCst);
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        });

        match ready_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Some(thread_id)) => Some(Self {
                stop,
                thread_id,
                join: Some(join),
            }),
            _ => {
                stop.store(true, Ordering::Relaxed);
                let _ = join.join();
                None
            }
        }
    }
}

impl Drop for DragGhost {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 == WM_MOUSEMOVE as usize {
        let handle = GHOST_HWND.load(Ordering::Relaxed);
        if handle != 0 {
            let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
            let hwnd = HWND(handle as *mut c_void);
            position_ghost(hwnd, predict_target(info.pt));
            let _ = SetTimer(hwnd, SETTLE_TIMER_ID, SETTLE_MS, None);
        }
    }
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

fn predict_target(cursor: POINT) -> POINT {
    let now = Instant::now();
    if let Some((last_x, last_y, last_at)) =
        LAST_MOTION.with(|motion| motion.replace(Some((cursor.x, cursor.y, now))))
    {
        let elapsed_ms = now.duration_since(last_at).as_secs_f32() * 1000.0;
        if elapsed_ms >= 0.5 && elapsed_ms < 100.0 {
            let raw_x = (cursor.x - last_x) as f32 / elapsed_ms;
            let raw_y = (cursor.y - last_y) as f32 / elapsed_ms;
            VELOCITY.with(|velocity| {
                let (old_x, old_y) = velocity.get();
                velocity.set((
                    old_x * 0.5 + raw_x * 0.5,
                    old_y * 0.5 + raw_y * 0.5,
                ));
            });
        } else if elapsed_ms >= 100.0 {
            VELOCITY.with(|velocity| velocity.set((0.0, 0.0)));
        }
    }

    let (velocity_x, velocity_y) = VELOCITY.with(|velocity| velocity.get());
    let lead_x = (velocity_x * LEAD_MS).clamp(-MAX_LEAD, MAX_LEAD) as i32;
    let lead_y = (velocity_y * LEAD_MS).clamp(-MAX_LEAD, MAX_LEAD) as i32;
    POINT {
        x: cursor.x + lead_x,
        y: cursor.y + lead_y,
    }
}

unsafe fn position_ghost(hwnd: HWND, target: POINT) {
    let _ = SetWindowPos(
        hwnd,
        HWND::default(),
        target.x + GHOST_OFFSET_X,
        target.y + GHOST_OFFSET_Y,
        0,
        0,
        SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER | SWP_SHOWWINDOW,
    );
}

unsafe fn move_to_cursor(hwnd: HWND) {
    let mut cursor = POINT::default();
    if GetCursorPos(&mut cursor).is_ok() {
        position_ghost(hwnd, cursor);
    }
}

unsafe extern "system" fn ghost_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCHITTEST {
        return LRESULT(HTTRANSPARENT as isize);
    }
    if msg == WM_TIMER && wparam.0 == SETTLE_TIMER_ID {
        VELOCITY.with(|velocity| velocity.set((0.0, 0.0)));
        LAST_MOTION.with(|motion| {
            let _ = motion.replace(None);
        });
        let mut cursor = POINT::default();
        if GetCursorPos(&mut cursor).is_ok() {
            position_ghost(hwnd, cursor);
        }
        let _ = KillTimer(hwnd, SETTLE_TIMER_ID);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn create_ghost_window(pixels: &image::RgbaImage) -> Option<HWND> {
    let hmodule = GetModuleHandleW(None).ok()?;
    let hinstance = HINSTANCE(hmodule.0);
    let class_name = w!("SnapCordDragGhost");

    let window_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(ghost_wndproc),
        hInstance: hinstance,
        lpszClassName: class_name,
        ..Default::default()
    };
    RegisterClassW(&window_class);

    let (width, height) = (pixels.width() as i32, pixels.height() as i32);
    let mut cursor = POINT::default();
    let _ = GetCursorPos(&mut cursor);

    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
        class_name,
        w!(""),
        WS_POPUP,
        cursor.x + GHOST_OFFSET_X,
        cursor.y + GHOST_OFFSET_Y,
        width,
        height,
        HWND::default(),
        HMENU::default(),
        hinstance,
        None,
    )
    .ok()?;

    if paint_ghost(hwnd, pixels).is_err() {
        let _ = DestroyWindow(hwnd);
        return None;
    }
    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    Some(hwnd)
}

unsafe fn paint_ghost(hwnd: HWND, pixels: &image::RgbaImage) -> windows::core::Result<()> {
    let (width, height) = (pixels.width() as i32, pixels.height() as i32);
    let screen = GetDC(HWND::default());

    let mut info = BITMAPINFO::default();
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };

    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap =
        match CreateDIBSection(screen, &info, DIB_RGB_COLORS, &mut bits, HANDLE::default(), 0) {
            Ok(bitmap) => bitmap,
            Err(err) => {
                ReleaseDC(HWND::default(), screen);
                return Err(err);
            }
        };
    copy_premultiplied(pixels, bits);

    let mem = CreateCompatibleDC(screen);
    let old = SelectObject(mem, bitmap);

    let mut dst = POINT::default();
    let _ = GetCursorPos(&mut dst);
    dst.x += GHOST_OFFSET_X;
    dst.y += GHOST_OFFSET_Y;
    let size = SIZE {
        cx: width,
        cy: height,
    };
    let src = POINT::default();
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };

    let result = UpdateLayeredWindow(
        hwnd,
        screen,
        Some(&dst as *const POINT),
        Some(&size as *const SIZE),
        mem,
        Some(&src as *const POINT),
        COLORREF(0),
        Some(&blend as *const BLENDFUNCTION),
        ULW_ALPHA,
    );

    SelectObject(mem, old);
    let _ = DeleteObject(bitmap);
    let _ = DeleteDC(mem);
    ReleaseDC(HWND::default(), screen);
    result
}

fn copy_premultiplied(pixels: &image::RgbaImage, target: *mut c_void) {
    let out = target as *mut u8;
    for (index, pixel) in pixels.pixels().enumerate() {
        let [r, g, b, a] = pixel.0;
        let alpha = a as u16;
        let premul = |channel: u8| ((channel as u16 * alpha + 127) / 255) as u8;
        let offset = index * 4;
        unsafe {
            *out.add(offset) = premul(b);
            *out.add(offset + 1) = premul(g);
            *out.add(offset + 2) = premul(r);
            *out.add(offset + 3) = a;
        }
    }
}

fn preview_from_thumbnail(thumbnail: &str) -> Option<image::DynamicImage> {
    let encoded = thumbnail.strip_prefix("data:image/")?;
    let (_, data) = encoded.split_once(";base64,")?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(data).ok()?;
    image::load_from_memory(&bytes).ok()
}

#[command]
pub fn start_file_drag(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let (path, thumbnail) = {
        let images = state.images.lock();
        images
            .iter()
            .find(|img| img.id == id)
            .map(|img| (img.path.clone(), img.thumbnail.clone()))
    }
    .ok_or_else(|| "图片不存在".to_string())?;

    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("文件不存在".to_string());
    }

    let preview = preview_from_thumbnail(&thumbnail)
        .or_else(|| image::open(&path_buf).ok())
        .map(|img| img.thumbnail(DRAG_THUMB_MAX, DRAG_THUMB_MAX))
        .ok_or_else(|| "图片读取失败".to_string())?;

    let ghost = DragGhost::start(preview.to_rgba8());

    // 拖拽过程中禁用点击穿透，避免窗口在指针下方频繁切换吸收状态。
    state.drag_active.store(true, Ordering::Relaxed);
    let result = crate::services::ole_drag::drag_files(&[path_buf]).map_err(|err| err.to_string());
    state.drag_active.store(false, Ordering::Relaxed);

    drop(ghost);
    result
}
