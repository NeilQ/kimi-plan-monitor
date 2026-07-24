// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;

use api::UsageData;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent,
};

// 全局状态
struct AppState {
    usage_data: Arc<Mutex<Option<UsageData>>>,
    token: Arc<Mutex<String>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn refresh_usage(state: State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let token = state.token.lock().unwrap().clone();

    if token.is_empty() {
        return Err("Token not set".to_string());
    }

    match api::fetch_usage(&token).await {
        Ok(data) => {
            let mut usage_data = state.usage_data.lock().unwrap();
            *usage_data = Some(data.clone());

            // 通知前端数据更新
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.emit("usage-updated", data);
            }

            Ok(())
        }
        Err(e) => Err(format!("Failed to fetch usage: {}", e)),
    }
}

// 启动定时轮询任务
fn start_polling_task(app_handle: tauri::AppHandle, state: Arc<Mutex<Option<UsageData>>>, token: Arc<Mutex<String>>) {
    tauri::async_runtime::spawn(async move {
        // 启动时立即执行一次
        let token_value = token.lock().unwrap().clone();
        if !token_value.is_empty() {
            if let Ok(data) = api::fetch_usage(&token_value).await {
                let mut usage_data = state.lock().unwrap();
                *usage_data = Some(data.clone());

                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("usage-updated", data);
                }
            }
        }

        // 每 60 秒轮询一次
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            let token_value = token.lock().unwrap().clone();
            if token_value.is_empty() {
                continue;
            }

            if let Ok(data) = api::fetch_usage(&token_value).await {
                let mut usage_data = state.lock().unwrap();
                *usage_data = Some(data.clone());

                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("usage-updated", data);
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let usage_data = Arc::new(Mutex::new(None));
    let token = Arc::new(Mutex::new("***REDACTED-TOKEN***".to_string()));

    let app_state = AppState {
        usage_data: usage_data.clone(),
        token: token.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(app_state)
        .setup(|app| {
            let usage_data = usage_data.clone();
            let token = token.clone();

            // 创建托盘菜单
            let show_item = MenuItem::with_id(app, "show", "显示/隐藏", true, None::<&str>)?;
            let refresh_item = MenuItem::with_id(app, "refresh", "立即刷新", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &refresh_item, &settings_item, &quit_item])?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Kimi Plan Monitor")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    "refresh" => {
                        let _ = app.emit("manual-refresh", ());
                    }
                    "settings" => {
                        let _ = app.emit("open-settings", ());
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // 启动定时轮询任务
            let app_handle = app.handle().clone();
            start_polling_task(app_handle, usage_data.clone(), token.clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            // 阻止窗口关闭，改为隐藏到托盘
            if let WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![greet, refresh_usage])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
