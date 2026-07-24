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

#[derive(Clone, Copy, PartialEq)]
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

// 检测窗口是否贴边
pub fn check_edge(window: &WebviewWindow) -> Option<Edge> {
    let monitor = window.current_monitor().ok()??;
    let screen_size = monitor.size();
    let screen_pos = monitor.position();

    let win_pos = window.outer_position().ok()?;
    let win_size = window.outer_size().ok()?;

    const THRESHOLD: i32 = 10; // 贴边阈值（像素）

    // 检测左边缘
    if (win_pos.x - screen_pos.x).abs() < THRESHOLD {
        return Some(Edge::Left);
    }

    // 检测右边缘
    if ((win_pos.x + win_size.width as i32) - (screen_pos.x + screen_size.width as i32)).abs() < THRESHOLD {
        return Some(Edge::Right);
    }

    // 检测上边缘
    if (win_pos.y - screen_pos.y).abs() < THRESHOLD {
        return Some(Edge::Top);
    }

    // 检测下边缘
    if ((win_pos.y + win_size.height as i32) - (screen_pos.y + screen_size.height as i32)).abs() < THRESHOLD {
        return Some(Edge::Bottom);
    }

    None
}

// 收起窗口
pub async fn hide_window(window: &WebviewWindow, edge: Edge, state: Arc<Mutex<AutoHideState>>) {
    let mut state = state.lock().unwrap();

    if state.is_hidden {
        return;
    }

    // 保存原始尺寸和位置
    if let (Ok(size), Ok(pos)) = (window.outer_size(), window.outer_position()) {
        state.original_size = Some(size);
        state.original_position = Some(pos);
        state.is_hidden = true;
        state.edge = Some(edge);

        // 根据边缘方向收起
        match edge {
            Edge::Left => {
                let _ = window.set_size(PhysicalSize::new(4, size.height));
            }
            Edge::Right => {
                let monitor = window.current_monitor().unwrap().unwrap();
                let screen_width = monitor.size().width;
                let _ = window.set_size(PhysicalSize::new(4, size.height));
                let _ = window.set_position(PhysicalPosition::new(
                    screen_width as i32 - 4,
                    pos.y,
                ));
            }
            Edge::Top => {
                let _ = window.set_size(PhysicalSize::new(size.width, 4));
            }
            Edge::Bottom => {
                let monitor = window.current_monitor().unwrap().unwrap();
                let screen_height = monitor.size().height;
                let _ = window.set_size(PhysicalSize::new(size.width, 4));
                let _ = window.set_position(PhysicalPosition::new(
                    pos.x,
                    screen_height as i32 - 4,
                ));
            }
        }
    }
}

// 展开窗口
#[allow(dead_code)]
pub async fn show_window(window: &WebviewWindow, state: Arc<Mutex<AutoHideState>>) {
    let mut state = state.lock().unwrap();

    if !state.is_hidden {
        return;
    }

    if let (Some(size), Some(pos)) = (state.original_size, state.original_position) {
        let _ = window.set_size(size);
        let _ = window.set_position(pos);
        state.is_hidden = false;
        state.edge = None;
    }
}

// 启动贴边检测任务
pub fn start_auto_hide_task(window: WebviewWindow, state: Arc<Mutex<AutoHideState>>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let enabled = {
                let state = state.lock().unwrap();
                state.enabled
            };

            if !enabled {
                continue;
            }

            let is_hidden = {
                let state = state.lock().unwrap();
                state.is_hidden
            };

            if !is_hidden {
                // 检测是否贴边
                if let Some(edge) = check_edge(&window) {
                    // 延迟 1 秒后收起
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    // 再次检查是否仍然贴边
                    if let Some(current_edge) = check_edge(&window) {
                        if current_edge == edge {
                            hide_window(&window, edge, state.clone()).await;
                        }
                    }
                }
            }
        }
    });
}
