use log::{debug, info, warn};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

// 贴边收起状态
pub struct AutoHideState {
    pub enabled: bool,
    pub is_hidden: bool,
    pub edge: Option<Edge>,
    // inner_size（物理像素）：set_size 控制的是内部尺寸，保存/恢复必须用同一个量
    pub original_size: Option<PhysicalSize<u32>>,
    // outer_position（物理像素）：set_position 控制的是外部位置
    pub original_position: Option<PhysicalPosition<i32>>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl AutoHideState {
    pub fn new() -> Self {
        Self {
            enabled: true,
            is_hidden: false,
            edge: None,
            original_size: None,
            original_position: None,
        }
    }
}

const EDGE_THRESHOLD: i32 = 8; // 贴边阈值（物理像素，基于可见框架）
const COLLAPSED_SIZE: u32 = 32; // 收起后的边宽（物理像素）
const HIDE_DELAY_MS: u64 = 800; // 贴边后延迟收起
const POLL_INTERVAL_MS: u64 = 200; // 轮询间隔
const HOVER_EXPAND_DIST: i32 = 30; // 鼠标距离小边多远时触发展开

// 检测窗口是否贴边
// 用 DwmGetWindowAttribute 拿可见框架边界，避免 Windows 不可见 resize 边框（~7px）
// 导致 outer_size 与视觉位置错位、检测失败
pub fn check_edge(window: &WebviewWindow) -> Option<Edge> {
    let monitor = window.current_monitor().ok()??;
    let screen_size = monitor.size();
    let screen_pos = monitor.position();

    let (win_pos, win_size) = visible_frame(window)?;

    let screen_left = screen_pos.x;
    let screen_right = screen_pos.x + screen_size.width as i32;
    let screen_top = screen_pos.y;
    let screen_bottom = screen_pos.y + screen_size.height as i32;

    let win_left = win_pos.0;
    let win_right = win_pos.0 + win_size.0;
    let win_top = win_pos.1;
    let win_bottom = win_pos.1 + win_size.1;

    debug!(
        "check_edge: screen=({},{})-({},{}), visible_win=({},{})-({},{})",
        screen_left, screen_top, screen_right, screen_bottom,
        win_left, win_top, win_right, win_bottom
    );

    // 检测左边缘
    if (win_left - screen_left).abs() <= EDGE_THRESHOLD {
        return Some(Edge::Left);
    }

    // 检测右边缘
    if (win_right - screen_right).abs() <= EDGE_THRESHOLD {
        return Some(Edge::Right);
    }

    // 检测上边缘
    if (win_top - screen_top).abs() <= EDGE_THRESHOLD {
        return Some(Edge::Top);
    }

    // 检测下边缘
    if (win_bottom - screen_bottom).abs() <= EDGE_THRESHOLD {
        return Some(Edge::Bottom);
    }

    None
}

// Windows 平台：用 DWM API 拿"用户实际看到的"窗口框架边界（物理像素）
#[cfg(target_os = "windows")]
fn visible_frame(window: &WebviewWindow) -> Option<((i32, i32), (i32, i32))> {
    use std::mem::MaybeUninit;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct RECT {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    const DWMWA_EXTENDED_FRAME_BOUNDS: u32 = 9;

    extern "system" {
        fn DwmGetWindowAttribute(
            hwnd: isize,
            dw_attribute: u32,
            pv_attribute: *mut core::ffi::c_void,
            cb_attribute: u32,
        ) -> i32;
    }

    let hwnd = window.hwnd().ok()?.0 as isize;

    let rect: RECT = unsafe {
        let mut r: MaybeUninit<RECT> = MaybeUninit::uninit();
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            r.as_mut_ptr() as *mut _,
            std::mem::size_of::<RECT>() as u32,
        );
        if hr != 0 {
            return None;
        }
        r.assume_init()
    };

    Some((
        (rect.left, rect.top),
        (rect.right - rect.left, rect.bottom - rect.top),
    ))
}

// 非 Windows：退化为 outer_position/size
#[cfg(not(target_os = "windows"))]
fn visible_frame(window: &WebviewWindow) -> Option<((i32, i32), (i32, i32))> {
    let pos = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    Some(((pos.x, pos.y), (size.width as i32, size.height as i32)))
}

// 收起窗口
pub fn hide_window(window: &WebviewWindow, edge: Edge, state: &Arc<Mutex<AutoHideState>>) {
    info!("hide_window: edge={:?}", edge);

    // set_size 控制内部尺寸 → 保存 inner_size；set_position 控制外部位置 → 保存 outer_position
    let (size, pos) = match (window.inner_size(), window.outer_position()) {
        (Ok(s), Ok(p)) => (s, p),
        (e1, e2) => {
            warn!("hide_window: failed to get size/pos: {:?} {:?}", e1, e2);
            return;
        }
    };

    let monitor = match window.current_monitor() {
        Ok(Some(m)) => m,
        _ => {
            warn!("hide_window: no monitor");
            return;
        }
    };
    let screen_size = monitor.size();
    let screen_pos = monitor.position();
    let screen_right = screen_pos.x + screen_size.width as i32;
    let screen_bottom = screen_pos.y + screen_size.height as i32;

    {
        let mut s = state.lock().unwrap();
        if s.is_hidden {
            return;
        }
        s.original_size = Some(size);
        s.original_position = Some(pos);
        s.is_hidden = true;
        s.edge = Some(edge);
    }

    // 关键：先禁用 resizable，否则 Windows 会强制最小窗口宽度
    let _ = window.set_resizable(false);

    // 全程物理像素：set_size/set_position 直接接受 PhysicalSize/PhysicalPosition
    match edge {
        Edge::Left => {
            let _ = window.set_size(PhysicalSize::new(COLLAPSED_SIZE, size.height));
            let _ = window.set_position(PhysicalPosition::new(screen_pos.x, pos.y));
        }
        Edge::Right => {
            let _ = window.set_size(PhysicalSize::new(COLLAPSED_SIZE, size.height));
            let _ = window.set_position(PhysicalPosition::new(
                screen_right - COLLAPSED_SIZE as i32,
                pos.y,
            ));
        }
        Edge::Top => {
            let _ = window.set_size(PhysicalSize::new(size.width, COLLAPSED_SIZE));
            let _ = window.set_position(PhysicalPosition::new(pos.x, screen_pos.y));
        }
        Edge::Bottom => {
            let _ = window.set_size(PhysicalSize::new(size.width, COLLAPSED_SIZE));
            let _ = window.set_position(PhysicalPosition::new(
                pos.x,
                screen_bottom - COLLAPSED_SIZE as i32,
            ));
        }
    }

    info!("hide_window: done");
}

// 展开窗口
pub fn show_window(window: &WebviewWindow, state: &Arc<Mutex<AutoHideState>>) {
    info!("show_window");

    let (size, pos) = {
        let mut s = state.lock().unwrap();
        if !s.is_hidden {
            return;
        }
        match (s.original_size, s.original_position) {
            (Some(sz), Some(p)) => {
                s.is_hidden = false;
                s.edge = None;
                (sz, p)
            }
            _ => return,
        }
    };

    // 全程物理像素、inner/outer 各自对应：保存时什么样，恢复就什么样，不因 DPI 错位
    let _ = window.set_resizable(true);
    let _ = window.set_size(size);
    let _ = window.set_position(pos);

    info!("show_window: done");
}

// 获取鼠标位置（Windows 平台）
#[cfg(target_os = "windows")]
fn get_cursor_pos() -> Option<(i32, i32)> {
    use std::mem::MaybeUninit;

    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    extern "system" {
        fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    }

    unsafe {
        let mut point: MaybeUninit<POINT> = MaybeUninit::uninit();
        if GetCursorPos(point.as_mut_ptr()) != 0 {
            let p = point.assume_init();
            Some((p.x, p.y))
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_cursor_pos() -> Option<(i32, i32)> {
    None
}

// 判断鼠标是否靠近收起的小边
fn is_cursor_near_hidden_edge(window: &WebviewWindow, edge: Edge) -> bool {
    let (cursor_x, cursor_y) = match get_cursor_pos() {
        Some(p) => p,
        None => return false,
    };

    let monitor = match window.current_monitor() {
        Ok(Some(m)) => m,
        _ => return false,
    };
    let screen_size = monitor.size();
    let screen_pos = monitor.position();

    let win_pos = match window.outer_position() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let win_size = match window.outer_size() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let screen_left = screen_pos.x;
    let screen_right = screen_pos.x + screen_size.width as i32;
    let screen_top = screen_pos.y;
    let screen_bottom = screen_pos.y + screen_size.height as i32;

    match edge {
        Edge::Left => {
            let in_x_range = cursor_x >= screen_left && cursor_x <= screen_left + HOVER_EXPAND_DIST;
            let in_y_range = cursor_y >= win_pos.y - 50 && cursor_y <= win_pos.y + win_size.height as i32 + 50;
            in_x_range && in_y_range
        }
        Edge::Right => {
            let in_x_range = cursor_x >= screen_right - HOVER_EXPAND_DIST && cursor_x <= screen_right;
            let in_y_range = cursor_y >= win_pos.y - 50 && cursor_y <= win_pos.y + win_size.height as i32 + 50;
            in_x_range && in_y_range
        }
        Edge::Top => {
            let in_y_range = cursor_y >= screen_top && cursor_y <= screen_top + HOVER_EXPAND_DIST;
            let in_x_range = cursor_x >= win_pos.x - 50 && cursor_x <= win_pos.x + win_size.width as i32 + 50;
            in_x_range && in_y_range
        }
        Edge::Bottom => {
            let in_y_range = cursor_y >= screen_bottom - HOVER_EXPAND_DIST && cursor_y <= screen_bottom;
            let in_x_range = cursor_x >= win_pos.x - 50 && cursor_x <= win_pos.x + win_size.width as i32 + 50;
            in_x_range && in_y_range
        }
    }
}

// 判断鼠标是否在窗口内（用于决定是否延迟收起）
fn is_cursor_in_window(window: &WebviewWindow) -> bool {
    let (cursor_x, cursor_y) = match get_cursor_pos() {
        Some(p) => p,
        None => return false,
    };

    let win_pos = match window.outer_position() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let win_size = match window.outer_size() {
        Ok(s) => s,
        Err(_) => return false,
    };

    cursor_x >= win_pos.x
        && cursor_x <= win_pos.x + win_size.width as i32
        && cursor_y >= win_pos.y
        && cursor_y <= win_pos.y + win_size.height as i32
}

// 启动贴边检测任务
pub fn start_auto_hide_task(window: WebviewWindow, state: Arc<Mutex<AutoHideState>>) {
    info!("start_auto_hide_task: started");

    tauri::async_runtime::spawn(async move {
        // 贴边状态计时（用于延迟收起）
        let mut edge_since: Option<std::time::Instant> = None;
        let mut tick = 0u32;

        loop {
            tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
            tick += 1;

            let (enabled, is_hidden, edge) = {
                let s = state.lock().unwrap();
                (s.enabled, s.is_hidden, s.edge)
            };

            // 每 5 秒输出一次心跳日志，确认任务在跑
            if tick % 25 == 0 {
                debug!(
                    "auto_hide tick: enabled={}, is_hidden={}, edge={:?}",
                    enabled, is_hidden, edge
                );
            }

            if !enabled {
                edge_since = None;
                continue;
            }

            if is_hidden {
                // 已收起：检测鼠标是否靠近边缘，是则展开
                if let Some(e) = edge {
                    if is_cursor_near_hidden_edge(&window, e) {
                        info!("auto_hide: cursor near edge, expanding");
                        show_window(&window, &state);
                        edge_since = None;
                    }
                }
            } else {
                // 未收起：检测是否贴边
                if let Some(detected_edge) = check_edge(&window) {
                    // 鼠标在窗口内时不收起
                    if is_cursor_in_window(&window) {
                        edge_since = None;
                        continue;
                    }

                    // 贴边计时
                    match edge_since {
                        None => {
                            info!("auto_hide: edge detected {:?}, start timing", detected_edge);
                            edge_since = Some(std::time::Instant::now());
                        }
                        Some(t) if t.elapsed().as_millis() >= HIDE_DELAY_MS as u128 => {
                            info!("auto_hide: hide delay elapsed, collapsing");
                            hide_window(&window, detected_edge, &state);
                            edge_since = None;
                        }
                        _ => {}
                    }
                } else {
                    // 离开边缘，重置计时
                    if edge_since.is_some() {
                        debug!("auto_hide: left edge, reset timing");
                    }
                    edge_since = None;
                }
            }
        }
    });
}
