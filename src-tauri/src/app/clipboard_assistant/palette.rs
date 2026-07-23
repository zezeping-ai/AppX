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
const PALETTE_ROUTE: &str = "/#/clipboard-palette";
/// 打开后短暂忽略点外关闭，避免热键同一次点击误关。
const DISMISS_ARM_DELAY: Duration = Duration::from_millis(200);
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
            if event.state == ShortcutState::Pressed {
                if is_session_locked(app) || !super::settings::is_palette_enabled() {
                    return;
                }
                let _ = toggle_palette_window(app);
            }
        })
        .map_err(|err| format!("注册剪切助手快捷键失败：{err}"))?;

    Ok(())
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
        present_existing(&window, app)?;
        return Ok(());
    }

    let app_handle = app.clone();
    app.run_on_main_thread(move || {
        if let Err(err) = create_palette_window(&app_handle) {
            eprintln!("[clipboard_palette] create window failed: {err}");
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

pub fn save_palette_geometry(app: &AppHandle, geometry: palette_geometry::PaletteGeometry) -> Result<(), String> {
    palette_geometry::write_geometry(app, &geometry)
}

fn present_existing(window: &WebviewWindow, app: &AppHandle) -> Result<(), String> {
    navigate_palette(window)?;
    #[cfg(target_os = "macos")]
    present_macos(window, app)?;
    #[cfg(not(target_os = "macos"))]
    present_other(window, app)?;
    Ok(())
}

fn create_palette_window(app: &AppHandle) -> Result<(), String> {
    let data_dir = webview_data_dir(app)?;
    std::fs::create_dir_all(&data_dir).map_err(|err| format!("创建面板数据目录失败：{err}"))?;

    let settings = settings::read_settings(app)?;
    let layout = settings.palette_layout.as_str();
    let (width, height) = initial_size(app, &settings, layout);

    let window = WebviewWindowBuilder::new(app, PALETTE_WINDOW_LABEL, WebviewUrl::App("index.html".into()))
        .title("剪切助手")
        .inner_size(width, height)
        .resizable(true)
        .decorations(false)
        .shadow(true)
        .always_on_top(true)
        .visible(false)
        .skip_taskbar(true)
        .focused(false)
        .data_directory(data_dir)
        .build()
        .map_err(|err| err.to_string())?;

    #[cfg(target_os = "macos")]
    {
        present_macos(&window, app)?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        apply_geometry(app, &window)?;
        navigate_palette(&window)?;
        window.show().map_err(|err| err.to_string())?;
        window.set_focus().map_err(|err| err.to_string())?;
        attach_dismiss_on_blur(&window);
        notify_palette_open(&window);
        arm_dismiss_on_blur();
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn present_macos(window: &WebviewWindow, app: &AppHandle) -> Result<(), String> {
    let settings = settings::read_settings(app)?;
    let layout = settings.palette_layout.as_str();
    let thickness = edge_thickness(layout, &settings);
    crate::app::windows::clipboard_panel::ensure_panel(window)?;
    // 抢焦点前用鼠标所在屏 setFrame（勿在 show 后再用 mainScreen 重算）
    crate::app::windows::clipboard_panel::place_on_mouse_screen(window, layout, thickness)?;
    crate::app::windows::clipboard_panel::show_panel(window)?;
    crate::app::windows::clipboard_panel::attach_outside_click_dismiss(
        window,
        make_outside_click_callback(app),
    );
    notify_palette_open(window);
    arm_dismiss_on_blur();
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn present_other(window: &WebviewWindow, app: &AppHandle) -> Result<(), String> {
    apply_geometry(app, window)?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    notify_palette_open(window);
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
    // 本应用内呼出时未记录外部目标：把焦点还给主窗口，避免输入框丢焦
    if !focus_target::has_captured_target() {
        if let Some(main) = app.get_webview_window(MAIN_WINDOW_LABEL) {
            let _ = main.set_focus();
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
    let _ = window.eval(
        "setTimeout(() => window.dispatchEvent(new CustomEvent('appx:clipboard-palette-open')), 0);",
    );
}

fn navigate_palette(window: &WebviewWindow) -> Result<(), String> {
    window
        .eval(format!("window.location.replace({PALETTE_ROUTE:?})").as_str())
        .map_err(|err| err.to_string())
}
