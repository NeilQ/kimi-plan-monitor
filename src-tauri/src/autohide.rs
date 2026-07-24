use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

// 贴边收起状态
pub struct AutoHideState {
    pub enabled: bool,
    pub is_hidden: bool,
    pub edge: Option<Edge>,
    pub original_size: Option<PhysicalSize<u32>>,
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

const EDGE_THRESHOLD: i32 = 5; // 贴边阈值（像素）
const COLLAPSED_SIZE: u32 = 4; // 收起后的边宽
const HIDE_DELAY_MS: u64 = 800; // 贴边后延迟收起
const POLL_INTERVAL_MS: u64 = 200; // 轮询间隔
const HOVER_EXPAND_DIST: i32 = 30; // 鼠标距离小边多远时触发展开

// 检测窗口是否贴边
pub fn check_edge(window: &WebviewWindow) -> Option<Edge> {
    let monitor = window.current_monitor().ok()??;
    let screen_size = monitor.size();
    let screen_pos = monitor.position();

    let win_pos = window.outer_position().ok()?;
    let win_size = window.outer_size().ok()?;

    let screen_left = screen_pos.x;
    let screen_right = screen_pos.x + screen_size.width as i32;
    let screen_top = screen_pos.y;
    let screen_bottom = screen_pos.y + screen_size.height as i32;

    let win_left = win_pos.x;
    let win_right = win_pos.x + win_size.width as i32;
    let win_top = win_pos.y;
    let win_bottom = win_pos.y + win_size.height as i32;

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

// 收起窗口
pub fn hide_window(window: &WebviewWindow, edge: Edge, state: &Arc<Mutex<AutoHideState>>) {
    let (size, pos) = match (window.outer_size(), window.outer_position()) {
        (Ok(s), Ok(p)) => (s, p),
        _ => return,
    };

    let monitor = match window.current_monitor() {
        Ok(Some(m)) => m,
        _ => return,
    };
    let screen_size = monitor.size();
    let screen_pos = monitor.position();

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

    match edge {
        Edge::Left => {
            let _ = window.set_size(PhysicalSize::new(COLLAPSED_SIZE, size.height));
            let _ = window.set_position(PhysicalPosition::new(screen_pos.x, pos.y));
        }
        Edge::Right => {
            let _ = window.set_size(PhysicalSize::new(COLLAPSED_SIZE, size.height));
            let _ = window.set_position(PhysicalPosition::new(
                screen_pos.x + screen_size.width as i32 - COLLAPSED_SIZE as i32,
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
                screen_pos.y + screen_size.height as i32 - COLLAPSED_SIZE as i32,
            ));
        }
    }
}

// 展开窗口
pub fn show_window(window: &WebviewWindow, state: &Arc<Mutex<AutoHideState>>) {
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

    let _ = window.set_size(size);
    let _ = window.set_position(pos);
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

    // 检查鼠标是否在屏幕边缘附近 + 在窗口收起条的 y/x 范围内
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
    tauri::async_runtime::spawn(async move {
        // 贴边状态计时（用于延迟收起）
        let mut edge_since: Option<std::time::Instant> = None;

        loop {
            tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;

            let (enabled, is_hidden, edge) = {
                let s = state.lock().unwrap();
                (s.enabled, s.is_hidden, s.edge)
            };

            if !enabled {
                edge_since = None;
                continue;
            }

            if is_hidden {
                // 已收起：检测鼠标是否靠近边缘，是则展开
                if let Some(e) = edge {
                    if is_cursor_near_hidden_edge(&window, e) {
                        show_window(&window, &state);
                        edge_since = None;
                    }
                }
            } else {
                // 未收起：检测是否贴边
                if check_edge(&window).is_some() {
                    // 鼠标在窗口内时不收起
                    if is_cursor_in_window(&window) {
                        edge_since = None;
                        continue;
                    }

                    // 贴边计时
                    match edge_since {
                        None => {
                            edge_since = Some(std::time::Instant::now());
                        }
                        Some(t) if t.elapsed().as_millis() >= HIDE_DELAY_MS as u128 => {
                            if let Some(e) = check_edge(&window) {
                                hide_window(&window, e, &state);
                                edge_since = None;
                            }
                        }
                        _ => {}
                    }
                } else {
                    // 离开边缘，重置计时
                    edge_since = None;
                }
            }
        }
    });
}
