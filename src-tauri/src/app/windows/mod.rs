use std::{thread, time::Duration};
use tauri::{Manager, WebviewUrl};

use crate::app::app_lock;

const MAIN_WINDOW_LABEL: &str = "main";
const PREFERENCES_WINDOW_LABEL: &str = "preferences";
const PREFERENCES_ROUTE: &str = "/#/preferences";

fn webview_data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位 WebView 数据目录：{err}"))
        .map(|dir| dir.join("webview"))
}

fn ensure_data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = webview_data_dir(app)?;
    std::fs::create_dir_all(&dir).map_err(|err| format!("创建 WebView 数据目录失败：{err}"))?;
    Ok(dir)
}

fn show_and_focus(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    window.show()?;
    window.set_focus()?;
    Ok(())
}

fn navigate_to_route(window: &tauri::WebviewWindow, route: &str) -> tauri::Result<()> {
    window.eval(format!("window.location.replace({route:?})").as_str())?;
    Ok(())
}

async fn hide_main_window_safely(app: tauri::AppHandle) {
    for _ in 0..10 {
        let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
            return;
        };
        let _ = window.hide();
        if !window.is_visible().unwrap_or(false) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn show_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.show()?;
        window.set_focus()?;
    }
    let _ = app_lock::relock_on_show(app);
    Ok(())
}

pub fn show_preferences_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(PREFERENCES_WINDOW_LABEL) {
        navigate_to_route(&window, PREFERENCES_ROUTE)?;
        show_and_focus(&window)?;
        return Ok(());
    }

    let app_handle = app.clone();
    app.run_on_main_thread(move || {
        let result = (|| -> tauri::Result<()> {
            let data_dir = ensure_data_dir(&app_handle).map_err(|err| {
                tauri::Error::Io(std::io::Error::other(err))
            })?;

            tauri::WebviewWindowBuilder::new(
                &app_handle,
                PREFERENCES_WINDOW_LABEL,
                WebviewUrl::App("index.html".into()),
            )
            .title("偏好设置")
            .inner_size(860.0, 640.0)
            .min_inner_size(760.0, 560.0)
            .resizable(true)
            .visible(true)
            .data_directory(data_dir)
            .build()?;

            if let Some(window) = app_handle.get_webview_window(PREFERENCES_WINDOW_LABEL) {
                navigate_to_route(&window, PREFERENCES_ROUTE)?;
                show_and_focus(&window)?;
            }
            Ok(())
        })();

        if let Err(error) = result {
            eprintln!("show preferences window failed: {error}");
        }
    })?;

    Ok(())
}

pub fn schedule_preferences_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    let app_handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(30));
        let deferred = app_handle.clone();
        let _ = app_handle.run_on_main_thread(move || {
            if let Err(error) = show_preferences_window(&deferred) {
                eprintln!("deferred preferences open failed: {error}");
            }
        });
    });
    Ok(())
}

pub fn handle_close_requested(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        if window.label() != MAIN_WINDOW_LABEL {
            return;
        }
        api.prevent_close();
        let app = window.app_handle().clone();
        tauri::async_runtime::spawn(async move {
            hide_main_window_safely(app).await;
        });
    }
}

#[tauri::command]
pub fn window_show_preferences(app: tauri::AppHandle) -> Result<(), String> {
    schedule_preferences_window(&app).map_err(|err| err.to_string())
}
