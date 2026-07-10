mod commands;
mod settings;

use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::app::clipboard;
use crate::app::code_snippets::SnippetRegistry;
use crate::app::focus_target;
use crate::app::text_delivery;

pub use commands::*;
pub use settings::read_palette_settings;

const PALETTE_WINDOW_LABEL: &str = "snippet-palette";
const PALETTE_ROUTE: &str = "/#/snippet-palette";

pub fn register_shortcut(app: &AppHandle) -> Result<(), String> {
    let settings = settings::read_palette_settings(app)?;
    let parsed = Shortcut::try_from(settings.palette_shortcut.as_str())
        .map_err(|err| format!("命令面板快捷键无效：{err}"))?;

    app.global_shortcut()
        .on_shortcut(parsed, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = toggle_palette_window(app);
            }
        })
        .map_err(|err| format!("注册命令面板快捷键失败：{err}"))?;

    Ok(())
}

pub fn toggle_palette_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|err| err.to_string())?;
        } else {
            show_palette_window(app)?;
        }
        return Ok(());
    }
    show_palette_window(app)
}

pub fn show_palette_window(app: &AppHandle) -> Result<(), String> {
    let own_bundle_id = app.config().identifier.clone();
    focus_target::capture(&own_bundle_id);

    if let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) {
        navigate_palette(&window)?;
        window.show().map_err(|err| err.to_string())?;
        window.set_focus().map_err(|err| err.to_string())?;
        notify_palette_open(&window);
        return Ok(());
    }

    let app_handle = app.clone();
    app.run_on_main_thread(move || {
        if let Err(err) = create_palette_window(&app_handle) {
            eprintln!("[palette] create window failed: {err}");
        }
    })
    .map_err(|err| err.to_string())?;

    Ok(())
}

pub fn hide_palette_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) {
        window.hide().map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn create_palette_window(app: &AppHandle) -> Result<(), String> {
    let data_dir = webview_data_dir(app)?;
    std::fs::create_dir_all(&data_dir).map_err(|err| format!("创建面板数据目录失败：{err}"))?;

    let window = WebviewWindowBuilder::new(app, PALETTE_WINDOW_LABEL, WebviewUrl::App("index.html".into()))
        .title("代码段")
        .inner_size(520.0, 420.0)
        .resizable(false)
        .always_on_top(true)
        .visible(true)
        .skip_taskbar(true)
        .focused(true)
        .data_directory(data_dir)
        .build()
        .map_err(|err| err.to_string())?;

    navigate_palette(&window)?;
    notify_palette_open(&window);
    Ok(())
}

fn webview_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位 WebView 数据目录：{err}"))
        .map(|dir| dir.join("webview-palette"))
}

fn notify_palette_open(window: &tauri::WebviewWindow) {
    let _ = window.eval(
        "setTimeout(() => window.dispatchEvent(new CustomEvent('appx:palette-open')), 0);",
    );
}

fn navigate_palette(window: &tauri::WebviewWindow) -> Result<(), String> {
    window
        .eval(format!("window.location.replace({PALETTE_ROUTE:?})").as_str())
        .map_err(|err| err.to_string())
}

pub fn copy_palette_item(
    app: &AppHandle,
    id: i64,
    registry: &SnippetRegistry,
) -> Result<(), String> {
    let snapshot = registry.snapshot();
    let Some(entry) = snapshot.by_id.get(&id) else {
        return Err(format!("代码段 #{id} 不存在"));
    };

    clipboard::set_text_persist(&entry.content)?;
    hide_palette_window(app)?;
    focus_target::restore();
    Ok(())
}

pub fn insert_palette_item(
    app: &AppHandle,
    id: i64,
    registry: &SnippetRegistry,
) -> Result<(), String> {
    let snapshot = registry.snapshot();
    let Some(entry) = snapshot.by_id.get(&id) else {
        return Err(format!("代码段 #{id} 不存在"));
    };

    let content = entry.content.clone();
    hide_palette_window(app)?;
    focus_target::restore();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        text_delivery::insert_at_focus(&content);
    });
    Ok(())
}
