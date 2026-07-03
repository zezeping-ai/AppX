use std::thread;
use std::time::Duration;
use tauri::Manager;

const MAIN_WINDOW_LABEL: &str = "main";

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
