//! 剪切面板 macOS 实现：tauri-nspanel + Cocoa 定位。
//!
//! - 全屏：FullScreenAuxiliary；显示用 CanJoinAllSpaces，隐藏用 MoveToActiveSpace
//! - 定位：鼠标所在屏；默认整屏 frame；Accessibility 确认 Dock 挡边时才让位
//! - 显示：不 make_key，避免抢走原输入框焦点
//! - 点外部关闭：全局鼠标按下监听（不依赖 resign_key）
//!
//! 注意：`tauri_panel!` 会在本模块注入 NSPoint/NSRect/NSEvent 等 use，勿再重复导入。

use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use block2::RcBlock;
use dispatch::Queue;
use objc2_app_kit::{NSApp, NSApplicationActivationPolicy, NSEventMask, NSRunningApplication, NSScreen};
use objc2_foundation::NSString;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt,
};

use crate::app::platform::macos_ax::{application, element_frame};

tauri_panel! {
    panel!(ClipboardPalettePanel {
        config: {
            can_become_key_window: true,
            becomes_key_only_if_needed: true,
            is_floating_panel: true
        }
    })
}

/// Dock 条实际厚度下限（自动隐藏收起时通常更小）
const DOCK_MIN_THICKNESS: f64 = 28.0;
const DOCK_MAX_THICKNESS: f64 = 200.0;

/// NSEvent monitor 句柄仅在主线程取用；包装以放入 Mutex。
struct ClickMonitor(objc2::rc::Retained<AnyObject>);
// SAFETY: 仅通过本模块在主线程创建/销毁 monitor。
unsafe impl Send for ClickMonitor {}
unsafe impl Sync for ClickMonitor {}

struct ClickMonitors {
    global: Option<ClickMonitor>,
    local: Option<ClickMonitor>,
}

static OUTSIDE_CLICK_ARMED: AtomicBool = AtomicBool::new(false);
static ON_OUTSIDE_CLICK: OnceLock<Mutex<Option<Box<dyn Fn() + Send + Sync>>>> = OnceLock::new();
static CLICK_MONITORS: Mutex<ClickMonitors> = Mutex::new(ClickMonitors {
    global: None,
    local: None,
});

fn outside_click_slot() -> &'static Mutex<Option<Box<dyn Fn() + Send + Sync>>> {
    ON_OUTSIDE_CLICK.get_or_init(|| Mutex::new(None))
}

fn behavior_shown() -> CollectionBehavior {
    CollectionBehavior::new()
        .stationary()
        .can_join_all_spaces()
        .full_screen_auxiliary()
}

fn behavior_hidden() -> CollectionBehavior {
    CollectionBehavior::new()
        .stationary()
        .move_to_active_space()
        .full_screen_auxiliary()
}

/// 首次将窗口转为 NSPanel 并做基础配置。
pub fn ensure_panel(window: &WebviewWindow) -> Result<(), String> {
    let app = window.app_handle();
    let label = window.label();
    let panel = match app.get_webview_panel(label) {
        Ok(panel) => panel,
        Err(_) => window
            .to_panel::<ClipboardPalettePanel>()
            .map_err(|err| format!("转换为 NSPanel 失败：{err}"))?,
    };

    panel.set_level(PanelLevel::MainMenu.value() + 1);
    panel.set_floating_panel(true);
    panel.set_hides_on_deactivate(false);
    panel.set_becomes_key_only_if_needed(true);
    panel.set_style_mask(
        StyleMask::empty()
            .nonactivating_panel()
            .resizable()
            .into(),
    );
    panel.set_collection_behavior(behavior_hidden().into());
    Ok(())
}

/// 创建普通 WebviewWindow 前临时禁止激活，避免 `NSApp.hide()` 后 unhide 把主窗口带出。
/// 应用当前可见时不要用 Prohibited，否则会抢走主窗口焦点（解锁页/启动会坏掉）。
pub fn with_no_activate<R>(f: impl FnOnce() -> R) -> R {
    let mtm = MainThreadMarker::new().expect("with_no_activate 须在主线程调用");
    let app = NSApp(mtm);
    if !app.isHidden() {
        return f();
    }
    let original = app.activationPolicy();
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Prohibited);
    let result = f();
    let _ = app.setActivationPolicy(original);
    result
}

/// 应用处于 hide 时，无激活解隐藏并压住主窗口，否则首次浮层会“创建成功但看不见”。
pub fn ensure_app_ready_for_overlay(app: &AppHandle) {
    let mtm = MainThreadMarker::new().expect("ensure_app_ready_for_overlay 须在主线程调用");
    let ns_app = NSApp(mtm);
    if !ns_app.isHidden() {
        return;
    }
    ns_app.unhideWithoutActivation();
    order_out_main(app);
}

fn order_out_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Ok(ns_window) = window.ns_window() {
        let ns_window: &NSWindow = unsafe { &*ns_window.cast() };
        ns_window.orderOut(None);
    }
}

/// 抢焦点前：按鼠标所在屏 Cocoa frame 铺边缘面板。
pub fn place_on_mouse_screen(window: &WebviewWindow, layout: &str, thickness: f64) -> Result<(), String> {
    let window = window.clone();
    let layout = layout.to_string();
    run_on_main(move || place_on_mouse_screen_main(&window, &layout, thickness))
}

fn place_on_mouse_screen_main(window: &WebviewWindow, layout: &str, thickness: f64) -> Result<(), String> {
    let mtm = MainThreadMarker::new().ok_or_else(|| "需要在主线程定位面板".to_string())?;
    let screen = screen_under_mouse(mtm).ok_or_else(|| "无法解析鼠标所在屏幕".to_string())?;
    let bounds = placement_bounds(&screen, mtm);
    let thickness = thickness.max(1.0).min(bounds.size.height.max(1.0));

    let rect = match layout {
        "topPanel" => NSRect::new(
            NSPoint::new(
                bounds.origin.x,
                bounds.origin.y + bounds.size.height - thickness,
            ),
            NSSize::new(bounds.size.width, thickness),
        ),
        "leftPanel" => {
            let width = thickness.min(bounds.size.width);
            NSRect::new(
                NSPoint::new(bounds.origin.x, bounds.origin.y),
                NSSize::new(width, bounds.size.height),
            )
        }
        "rightPanel" => {
            let width = thickness.min(bounds.size.width);
            NSRect::new(
                NSPoint::new(bounds.origin.x + bounds.size.width - width, bounds.origin.y),
                NSSize::new(width, bounds.size.height),
            )
        }
        _ => NSRect::new(
            NSPoint::new(bounds.origin.x, bounds.origin.y),
            NSSize::new(bounds.size.width, thickness),
        ),
    };

    let ns_window = window
        .ns_window()
        .map_err(|err| format!("获取 NSWindow 失败：{err}"))?;
    let ns_window: &NSWindow = unsafe { &*ns_window.cast() };
    ns_window.setFrame_display(rect, false);
    Ok(())
}

/// 显示面板：只前置，不 make_key，保留原应用/输入框焦点。
pub fn show_panel(window: &WebviewWindow) -> Result<(), String> {
    let app = window.app_handle();
    ensure_app_ready_for_overlay(&app);
    let panel = app
        .get_webview_panel(window.label())
        .map_err(|_| "剪切面板尚未转换为 NSPanel".to_string())?;
    panel.show();
    panel.set_collection_behavior(behavior_shown().into());
    panel.order_front_regardless();
    Ok(())
}

/// 隐藏面板，并拆除点外关闭监听。
pub fn hide_panel(window: &WebviewWindow) {
    stop_outside_click_monitor();
    OUTSIDE_CLICK_ARMED.store(false, Ordering::Release);
    let app = window.app_handle();
    if let Ok(panel) = app.get_webview_panel(window.label()) {
        panel.hide();
        panel.set_collection_behavior(behavior_hidden().into());
        return;
    }
    let _ = window.hide();
}

/// 注册点外部关闭；展示后短延迟再 arm，避免热键同一次点击误关。
pub fn attach_outside_click_dismiss(
    window: &WebviewWindow,
    on_outside: impl Fn() + Send + Sync + 'static,
) {
    if let Ok(mut guard) = outside_click_slot().lock() {
        *guard = Some(Box::new(on_outside));
    }
    let window = window.clone();
    run_on_main(move || start_outside_click_monitor(&window));
}

pub fn arm_outside_click_dismiss() {
    OUTSIDE_CLICK_ARMED.store(true, Ordering::Release);
}

pub fn disarm_outside_click_dismiss() {
    OUTSIDE_CLICK_ARMED.store(false, Ordering::Release);
}

fn start_outside_click_monitor(window: &WebviewWindow) {
    stop_outside_click_monitor();

    let label = window.label().to_string();
    let app = window.app_handle().clone();
    let label_local = label.clone();
    let app_local = app.clone();

    // 全局：点到其他 App；本地：点到本进程主窗口等（全局监听收不到）
    let global_block = RcBlock::new(move |_event: NonNull<NSEvent>| {
        maybe_dismiss_outside_click(&app, &label);
    });
    let local_block = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
        maybe_dismiss_outside_click(&app_local, &label_local);
        event.as_ptr()
    });

    let mask = NSEventMask::LeftMouseDown | NSEventMask::RightMouseDown;
    let global = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(mask, &global_block);
    let local = unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &local_block) };
    if let Ok(mut slot) = CLICK_MONITORS.lock() {
        slot.global = global.map(ClickMonitor);
        slot.local = local.map(ClickMonitor);
    }
}

fn maybe_dismiss_outside_click(app: &tauri::AppHandle, label: &str) {
    if !OUTSIDE_CLICK_ARMED.load(Ordering::Acquire) {
        return;
    }
    let Some(webview) = app.get_webview_window(label) else {
        return;
    };
    if !webview.is_visible().unwrap_or(false) {
        return;
    }
    let Ok(ns_ptr) = webview.ns_window() else {
        return;
    };
    let ns_window: &NSWindow = unsafe { &*ns_ptr.cast() };
    let frame = ns_window.frame();
    let mouse = NSEvent::mouseLocation();
    let inside = mouse.x >= frame.origin.x
        && mouse.x <= frame.origin.x + frame.size.width
        && mouse.y >= frame.origin.y
        && mouse.y <= frame.origin.y + frame.size.height;
    if inside {
        return;
    }
    if let Ok(guard) = outside_click_slot().lock() {
        if let Some(cb) = guard.as_ref() {
            cb();
        }
    }
}

fn stop_outside_click_monitor() {
    if let Ok(mut slot) = CLICK_MONITORS.lock() {
        for monitor in [slot.global.take(), slot.local.take()].into_iter().flatten() {
            unsafe {
                NSEvent::removeMonitor(&monitor.0);
            }
        }
    }
}

/// 创建窗口时取鼠标屏尺寸。
pub fn mouse_screen_size() -> Option<(f64, f64)> {
    run_on_main(|| {
        let mtm = MainThreadMarker::new()?;
        let screen = screen_under_mouse(mtm)?;
        let bounds = placement_bounds(&screen, mtm);
        Some((bounds.size.width, bounds.size.height))
    })
}

/// 默认整屏；仅当 Accessibility 确认 Dock 条正挡边时才收缩（读不到则不让位）。
fn placement_bounds(screen: &NSScreen, mtm: MainThreadMarker) -> NSRect {
    let frame = screen.frame();
    let Some(dock_q) = dock_ax_list_rect() else {
        return frame;
    };
    let Some(screen_q) = cocoa_to_quartz(frame, mtm) else {
        return frame;
    };
    if !rects_intersect(&dock_q, &screen_q) {
        return frame;
    }

    let mut bounds = frame;
    let horizontal_strip = dock_q.h >= DOCK_MIN_THICKNESS
        && dock_q.h <= DOCK_MAX_THICKNESS
        && dock_q.w > dock_q.h * 1.5;
    let vertical_strip = dock_q.w >= DOCK_MIN_THICKNESS
        && dock_q.w <= DOCK_MAX_THICKNESS
        && dock_q.h > dock_q.w * 1.5;

    if horizontal_strip {
        let screen_bottom = screen_q.y + screen_q.h;
        let dock_bottom = dock_q.y + dock_q.h;
        if (screen_bottom - dock_bottom).abs() <= 16.0 {
            let reserve = (screen_bottom - dock_q.y).clamp(DOCK_MIN_THICKNESS, DOCK_MAX_THICKNESS);
            bounds.origin.y += reserve;
            bounds.size.height = (bounds.size.height - reserve).max(1.0);
            return bounds;
        }
        if (dock_q.y - screen_q.y).abs() <= 16.0 {
            let reserve = dock_q.h.clamp(DOCK_MIN_THICKNESS, DOCK_MAX_THICKNESS);
            bounds.size.height = (bounds.size.height - reserve).max(1.0);
            return bounds;
        }
    }

    if vertical_strip {
        if (dock_q.x - screen_q.x).abs() <= 16.0 {
            let reserve = (dock_q.x + dock_q.w - screen_q.x)
                .clamp(DOCK_MIN_THICKNESS, DOCK_MAX_THICKNESS);
            bounds.origin.x += reserve;
            bounds.size.width = (bounds.size.width - reserve).max(1.0);
            return bounds;
        }
        if ((dock_q.x + dock_q.w) - (screen_q.x + screen_q.w)).abs() <= 16.0 {
            let reserve = (screen_q.x + screen_q.w - dock_q.x)
                .clamp(DOCK_MIN_THICKNESS, DOCK_MAX_THICKNESS);
            bounds.size.width = (bounds.size.width - reserve).max(1.0);
            return bounds;
        }
    }

    frame
}

struct QuartzRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn dock_ax_list_rect() -> Option<QuartzRect> {
    use core_foundation::base::TCFType;
    use core_foundation::array::CFArray;

    let bundle = NSString::from_str("com.apple.dock");
    let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle);
    let dock = apps.firstObject()?;
    let pid = dock.processIdentifier();
    let app_el = application(pid)?;
    let children = crate::app::platform::macos_ax::copy_attr(app_el.as_ptr(), "AXChildren")?;
    let ptr = children.0;
    std::mem::forget(children);
    let array: CFArray = unsafe { TCFType::wrap_under_create_rule(ptr as _) };

    for child in array.get_all_values() {
        if child.is_null() {
            continue;
        }
        let Some((point, size)) = element_frame(child as _) else {
            continue;
        };
        if (size.height >= DOCK_MIN_THICKNESS && size.height <= DOCK_MAX_THICKNESS)
            || (size.width >= DOCK_MIN_THICKNESS && size.width <= DOCK_MAX_THICKNESS)
        {
            return Some(QuartzRect {
                x: point.x,
                y: point.y,
                w: size.width,
                h: size.height,
            });
        }
    }
    None
}

fn cocoa_to_quartz(rect: NSRect, mtm: MainThreadMarker) -> Option<QuartzRect> {
    let primary = NSScreen::screens(mtm).firstObject()?;
    let primary_frame = primary.frame();
    let flip_y = primary_frame.origin.y + primary_frame.size.height;
    Some(QuartzRect {
        x: rect.origin.x,
        y: flip_y - rect.origin.y - rect.size.height,
        w: rect.size.width,
        h: rect.size.height,
    })
}

fn rects_intersect(a: &QuartzRect, b: &QuartzRect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

fn screen_under_mouse(mtm: MainThreadMarker) -> Option<objc2::rc::Retained<NSScreen>> {
    let mouse = NSEvent::mouseLocation();
    for screen in NSScreen::screens(mtm) {
        let frame = screen.frame();
        if mouse.x >= frame.origin.x
            && mouse.x < frame.origin.x + frame.size.width
            && mouse.y >= frame.origin.y
            && mouse.y < frame.origin.y + frame.size.height
        {
            return Some(screen);
        }
    }
    NSScreen::mainScreen(mtm).or_else(|| NSScreen::screens(mtm).firstObject())
}

fn run_on_main<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
    if MainThreadMarker::new().is_some() {
        return f();
    }
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    Queue::main().exec_async(move || {
        let _ = tx.send(f());
    });
    rx.recv().expect("main thread dispatch")
}
