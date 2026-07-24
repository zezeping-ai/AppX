//! 剪切助手浮层窗口。
//!
//! 策略：启动时预创建隐藏 NSPanel，热键只做 show/hide，避免首次 `WebviewWindow::build`
//! 激活应用、主路由闪屏、以及 SPA 尚未就绪时 `location.replace` 的竞态。

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
#[cfg(not(target_os = "macos"))]
use tauri::WindowEvent;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::app::app_lock::is_session_locked;
use crate::app::clipboard_assistant::access::ensure_usable;
use crate::app::clipboard_assistant::palette_geometry;
use crate::app::clipboard_assistant::settings;
use crate::app::focus_target;

const PALETTE_WINDOW_LABEL: &str = "clipboard-palette";
/// 入口带 hash，避免先渲染 Layout/editor 再跳转。
const PALETTE_ENTRY: &str = "index.html#/clipboard-palette";
/// 打开后短暂忽略点外关闭，避免热键同一次点击误关。
const DISMISS_ARM_DELAY: Duration = Duration::from_millis(250);
const SIDE_PANEL_WIDTH: f64 = 320.0;
const MAIN_WINDOW_LABEL: &str = "main";

static DISMISS_ON_BLUR_ARMED: AtomicBool = AtomicBool::new(false);

pub fn register_shortcut(app: &AppHandle) -> Result<(), String> {
    if !super::settings::is_palette_enabled() {
        return Ok(());
    }

    let settings = settings::read_settings(app)?;
    let parsed = Shortcut::try_from(settings.palette_shortcut.as_str())
        .map_err(|err| format!("剪切助手快捷键无效：{err}"))?;

    app.global_shortcut()
        .on_shortcut(parsed, move |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            if is_session_locked(app) || !super::settings::is_palette_enabled() {
                return;
            }
            let _ = toggle_palette_window(app);
        })
        .map_err(|err| format!("注册剪切助手快捷键失败：{err}"))?;

    Ok(())
}

/// 启动/解锁后预创建隐藏浮层，使首次热键与后续路径一致。
/// 轻微延迟，避免与解锁页跳转/主窗口激活抢焦点。
pub fn ensure_window_ready(app: &AppHandle) {
    if app.get_webview_window(PALETTE_WINDOW_LABEL).is_some() {
        return;
    }
    let app_handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(400));
        if is_session_locked(&app_handle) {
            return;
        }
        let app_for_main = app_handle.clone();
        let _ = app_handle.run_on_main_thread(move || {
            if app_for_main.get_webview_window(PALETTE_WINDOW_LABEL).is_some() {
                return;
            }
            if let Err(err) = create_palette_window(&app_for_main, false) {
                eprintln!("[clipboard_palette] precreate failed: {err}");
            }
        });
    });
}

pub fn toggle_palette_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            disarm_dismiss_on_blur();
            save_geometry_if_needed(app, &window)?;
            hide_window(app, &window);
        } else {
            show_palette_window(app)?;
        }
        return Ok(());
    }
    show_palette_window(app)
}

pub fn show_palette_window(app: &AppHandle) -> Result<(), String> {
    ensure_usable(app)?;
    if !super::settings::is_palette_enabled() {
        return Ok(());
    }
    let own_bundle_id = app.config().identifier.clone();
    focus_target::capture(&own_bundle_id);

    if let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) {
        return present_window(&window, app);
    }

    // 预创建尚未完成时的兜底：主线程同步创建并展示
    let app_handle = app.clone();
    app.run_on_main_thread(move || {
        if let Some(window) = app_handle.get_webview_window(PALETTE_WINDOW_LABEL) {
            if let Err(err) = present_window(&window, &app_handle) {
                eprintln!("[clipboard_palette] present failed: {err}");
            }
            return;
        }
        match create_palette_window(&app_handle, true) {
            Ok(()) => {}
            Err(err) => eprintln!("[clipboard_palette] create+show failed: {err}"),
        }
    })
    .map_err(|err| err.to_string())?;

    Ok(())
}

pub fn hide_palette_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) {
        disarm_dismiss_on_blur();
        save_geometry_if_needed(app, &window)?;
        hide_window(app, &window);
    }
    Ok(())
}

pub fn save_palette_geometry(
    app: &AppHandle,
    geometry: palette_geometry::PaletteGeometry,
) -> Result<(), String> {
    palette_geometry::write_geometry(app, &geometry)
}

fn present_window(window: &WebviewWindow, app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    present_macos(window, app)?;
    #[cfg(not(target_os = "macos"))]
    present_other(window, app)?;
    notify_palette_open(window);
    Ok(())
}

/// `show_after_create`：预创建传 false；热键兜底创建传 true。
fn create_palette_window(app: &AppHandle, show_after_create: bool) -> Result<(), String> {
    let data_dir = webview_data_dir(app)?;
    std::fs::create_dir_all(&data_dir).map_err(|err| format!("创建面板数据目录失败：{err}"))?;

    let settings = settings::read_settings(app)?;
    let layout = settings.palette_layout.as_str();
    let (width, height) = initial_size(app, &settings, layout);

    let build = || {
        WebviewWindowBuilder::new(app, PALETTE_WINDOW_LABEL, WebviewUrl::App(PALETTE_ENTRY.into()))
            .title("剪切助手")
            .inner_size(width, height)
            .resizable(true)
            .decorations(false)
            .shadow(true)
            .always_on_top(true)
            .visible(false)
            .skip_taskbar(true)
            .focused(false)
            .data_directory(data_dir.clone())
            .build()
            .map_err(|err| err.to_string())
    };

    #[cfg(target_os = "macos")]
    let window = crate::app::windows::clipboard_panel::with_no_activate(|| {
        let window = build()?;
        crate::app::windows::clipboard_panel::ensure_panel(&window)?;
        Ok::<_, String>(window)
    })?;
    #[cfg(not(target_os = "macos"))]
    let window = build()?;

    #[cfg(not(target_os = "macos"))]
    attach_dismiss_on_blur(&window);

    if show_after_create {
        present_window(&window, app)?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn present_macos(window: &WebviewWindow, app: &AppHandle) -> Result<(), String> {
    let settings = settings::read_settings(app)?;
    let layout = settings.palette_layout.as_str();
    let thickness = edge_thickness(layout, &settings);
    crate::app::windows::clipboard_panel::ensure_panel(window)?;
    crate::app::windows::clipboard_panel::place_on_mouse_screen(window, layout, thickness)?;
    crate::app::windows::clipboard_panel::show_panel(window)?;
    crate::app::windows::clipboard_panel::attach_outside_click_dismiss(
        window,
        make_outside_click_callback(app),
    );
    arm_dismiss_on_blur();
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn present_other(window: &WebviewWindow, app: &AppHandle) -> Result<(), String> {
    apply_geometry(app, window)?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    arm_dismiss_on_blur();
    Ok(())
}

fn initial_size(
    app: &AppHandle,
    settings: &settings::ClipboardAssistantSettings,
    layout: &str,
) -> (f64, f64) {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        if let Some((sw, sh)) = crate::app::windows::clipboard_panel::mouse_screen_size() {
            return if layout == "leftPanel" || layout == "rightPanel" {
                (SIDE_PANEL_WIDTH, sh)
            } else {
                (sw, settings.palette_height.max(300) as f64)
            };
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(monitor) = app.primary_monitor().ok().flatten() {
            let rect = palette_geometry::compute_rect(settings, &monitor);
            return (rect.width, rect.height);
        }
    }
    (
        settings.palette_width.max(640) as f64,
        palette_geometry::inner_height_for_settings(settings),
    )
}

fn edge_thickness(layout: &str, settings: &settings::ClipboardAssistantSettings) -> f64 {
    if layout == "leftPanel" || layout == "rightPanel" {
        SIDE_PANEL_WIDTH
    } else {
        settings.palette_height.max(300) as f64
    }
}

fn hide_window(app: &AppHandle, window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    crate::app::windows::clipboard_panel::hide_panel(window);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.hide();
    }
    if !focus_target::has_captured_target() {
        if let Some(main) = app.get_webview_window(MAIN_WINDOW_LABEL) {
            if main.is_visible().unwrap_or(false) {
                let _ = main.set_focus();
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn make_outside_click_callback(app: &AppHandle) -> impl Fn() + Send + Sync + 'static {
    let app = app.clone();
    move || {
        if !DISMISS_ON_BLUR_ARMED.load(Ordering::Acquire) {
            return;
        }
        let Ok(settings) = settings::read_settings(&app) else {
            return;
        };
        if !settings.auto_hide_on_click_outside {
            return;
        }
        let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) else {
            return;
        };
        if !window.is_visible().unwrap_or(false) {
            return;
        }
        disarm_dismiss_on_blur();
        let _ = save_geometry_if_needed(&app, &window);
        hide_window(&app, &window);
    }
}

fn arm_dismiss_on_blur() {
    DISMISS_ON_BLUR_ARMED.store(false, Ordering::Release);
    #[cfg(target_os = "macos")]
    {
        thread::spawn(|| {
            thread::sleep(DISMISS_ARM_DELAY);
            crate::app::windows::clipboard_panel::arm_outside_click_dismiss();
            DISMISS_ON_BLUR_ARMED.store(true, Ordering::Release);
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        thread::spawn(|| {
            thread::sleep(DISMISS_ARM_DELAY);
            DISMISS_ON_BLUR_ARMED.store(true, Ordering::Release);
        });
    }
}

fn disarm_dismiss_on_blur() {
    DISMISS_ON_BLUR_ARMED.store(false, Ordering::Release);
    #[cfg(target_os = "macos")]
    crate::app::windows::clipboard_panel::disarm_outside_click_dismiss();
}

#[cfg(not(target_os = "macos"))]
fn attach_dismiss_on_blur(window: &WebviewWindow) {
    let app = window.app_handle().clone();
    window.on_window_event(move |event| {
        if !matches!(event, WindowEvent::Focused(false)) {
            return;
        }
        if !DISMISS_ON_BLUR_ARMED.load(Ordering::Acquire) {
            return;
        }
        let Ok(settings) = settings::read_settings(&app) else {
            return;
        };
        if !settings.auto_hide_on_click_outside {
            return;
        }
        let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) else {
            return;
        };
        if !window.is_visible().unwrap_or(false) {
            return;
        }
        disarm_dismiss_on_blur();
        let _ = save_geometry_if_needed(&app, &window);
        hide_window(&app, &window);
    });
}

#[cfg(not(target_os = "macos"))]
fn apply_geometry(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let settings = settings::read_settings(app)?;
    palette_geometry::apply_to_window(window, app, &settings)
}

fn save_geometry_if_needed(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let settings = settings::read_settings(app)?;
    if !settings.remember_window_position {
        return Ok(());
    }
    let geometry = palette_geometry::capture_from_window(window, settings.palette_layout.as_str())?;
    palette_geometry::write_geometry(app, &geometry)
}

pub fn refresh_palette_geometry(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(PALETTE_WINDOW_LABEL) {
        #[cfg(target_os = "macos")]
        {
            let settings = settings::read_settings(app)?;
            let layout = settings.palette_layout.as_str();
            let thickness = edge_thickness(layout, &settings);
            crate::app::windows::clipboard_panel::place_on_mouse_screen(&window, layout, thickness)?;
        }
        #[cfg(not(target_os = "macos"))]
        apply_geometry(app, &window)?;
        if window.is_visible().unwrap_or(false) {
            notify_palette_open(&window);
        }
    }
    Ok(())
}

fn webview_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位 WebView 数据目录：{err}"))
        .map(|dir| dir.join("webview-clipboard-palette"))
}

fn notify_palette_open(window: &WebviewWindow) {
    // 页面已在 clipboard-palette：只派发打开事件，不再 location.replace（避免闪屏/重载）
    let _ = window.eval(
        r#"(function () {
  var go = function () {
    window.dispatchEvent(new CustomEvent('appx:clipboard-palette-open'));
  };
  if (location.hash.indexOf('clipboard-palette') >= 0) {
    go();
    return;
  }
  location.replace('/#/clipboard-palette');
  setTimeout(go, 50);
})();"#,
    );
}
