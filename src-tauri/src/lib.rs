// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;
mod autohide;

use api::UsageData;
use autohide::AutoHideState;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_store::StoreExt;

// 全局状态
struct AppState {
    usage_data: Arc<Mutex<Option<UsageData>>>,
    token: Arc<Mutex<String>>,
    auto_hide_state: Arc<Mutex<AutoHideState>>,
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

#[tauri::command]
fn get_auto_hide_enabled(state: State<'_, AppState>) -> bool {
    state.auto_hide_state.lock().unwrap().enabled
}

#[tauri::command]
fn set_auto_hide_enabled(state: State<'_, AppState>, app_handle: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    state.auto_hide_state.lock().unwrap().enabled = enabled;

    // 持久化到 store
    let store = app_handle.store("config.json").map_err(|e| format!("failed to open store: {}", e))?;
    store.set("auto_hide_enabled", serde_json::json!(enabled));
    store.save().map_err(|e| format!("failed to save store: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_token(state: State<'_, AppState>) -> String {
    state.token.lock().unwrap().clone()
}

#[tauri::command]
fn set_token(state: State<'_, AppState>, app_handle: tauri::AppHandle, token: String) -> Result<(), String> {
    *state.token.lock().unwrap() = token.clone();

    // 持久化到 store
    let store = app_handle.store("config.json").map_err(|e| format!("failed to open store: {}", e))?;
    store.set("token", serde_json::json!(token));
    store.save().map_err(|e| format!("failed to save store: {}", e))?;

    // 保存成功后由后端异步刷新一次并推送给主窗口
    // 不让设置窗口自己 invoke refresh_usage，避免窗口关闭后 Promise 回传失败
    if !token.is_empty() {
        tauri::async_runtime::spawn(async move {
            if let Ok(data) = api::fetch_usage(&token).await {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("usage-updated", data);
                }
            }
        });
    }

    Ok(())
}

#[tauri::command]
fn get_config_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let path = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("failed to get app config dir: {}", e))?
        .join("config.json");
    Ok(path.to_string_lossy().to_string())
}

// 关闭当前调用窗口（后端关窗不受前端 ACL 权限限制）
#[tauri::command]
fn close_current_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| format!("failed to close window: {}", e))
}

// 激活窗口：若被遮挡、最小化或不可见则拉到前台
fn activate_window(window: &tauri::WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    // alwaysOnTop 已开启，再次显式置顶确保不被其他窗口压住
    let _ = window.set_always_on_top(true);
}

// 打开或激活设置窗口
fn open_settings_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        // 已存在：取消最小化并显示、置顶、聚焦
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_always_on_top(true);
        let _ = window.set_focus();
        Ok(())
    } else {
        // 不存在：动态创建，关闭即销毁
        let _window = WebviewWindowBuilder::new(app, "settings", tauri::WebviewUrl::App("settings.html".into()))
            .title("设置")
            .inner_size(420.0, 320.0)
            .decorations(true)
            .always_on_top(true)
            .resizable(false)
            .center()
            .skip_taskbar(false)
            .transparent(false)
            .visible(true)
            .build()
            .map_err(|e| format!("failed to create settings window: {}", e))?;
        Ok(())
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let usage_data = Arc::new(Mutex::new(None));
    let auto_hide_state = Arc::new(Mutex::new(AutoHideState::new()));

    // Token 从配置文件读取，初始为空，需用户在设置中填写
    let token = Arc::new(Mutex::new(String::new()));

    let app_state = AppState {
        usage_data: usage_data.clone(),
        token: token.clone(),
        auto_hide_state: auto_hide_state.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(app_state)
        .setup(move |app| {
            let usage_data = usage_data.clone();
            let token = token.clone();
            let auto_hide_state = auto_hide_state.clone();

            // 从 store 读取 Token
            let store = app.store("config.json").expect("failed to open store");
            if let Some(saved_token) = store.get("token") {
                if let Some(token_str) = saved_token.as_str() {
                    *token.lock().unwrap() = token_str.to_string();
                }
            }

            // 从 store 读取自动收起设置
            if let Some(enabled) = store.get("auto_hide_enabled") {
                if let Some(enabled_bool) = enabled.as_bool() {
                    auto_hide_state.lock().unwrap().enabled = enabled_bool;
                }
            }

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
                                activate_window(&window);
                            }
                        }
                    }
                    "refresh" => {
                        if let Some(window) = app.get_webview_window("main") {
                            activate_window(&window);
                        }
                        let _ = app.emit("manual-refresh", ());
                    }
                    "settings" => {
                        let _ = open_settings_window(app);
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 只在左键按下时切换显隐，避免右键菜单被一同触发关闭
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                activate_window(&window);
                            }
                        }
                    }
                })
                .build(app)?;

            // 启动定时轮询任务
            let app_handle = app.handle().clone();
            start_polling_task(app_handle, usage_data.clone(), token.clone());

            // 启动贴边自动收起任务
            if let Some(window) = app.get_webview_window("main") {
                autohide::start_auto_hide_task(window, auto_hide_state.clone());
            }

            // 首次启动未配置 Token 时，打开设置窗口
            if token.lock().unwrap().is_empty() {
                let app_handle = app.handle().clone();
                let _ = open_settings_window(&app_handle);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // 阻止主窗口关闭，改为隐藏到托盘；设置窗口允许正常关闭
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![greet, refresh_usage, get_auto_hide_enabled, set_auto_hide_enabled, get_token, set_token, get_config_path, close_current_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
