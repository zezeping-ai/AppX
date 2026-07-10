//! macOS 全屏关闭黑屏修复。
//!
//! 根因：在全屏 Space 中调用 hide/orderOut 会打断过渡动画，残留黑屏（tauri#10580 / #12056）。
//! 方案：先退出全屏，在 MainEventsCleared 中确认退出完成后再于主线程 orderOut。

use std::{
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApp, NSWindow};
use tauri::{Manager, RunEvent, WindowEvent};

const MAIN_WINDOW_LABEL: &str = "main";
const FULLSCREEN_HIDE_DELAY: Duration = Duration::from_millis(700);
const FULLSCREEN_EXIT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloseRequestAction {
    HideImmediately,
    ExitFullscreenThenHide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainEventsAction {
    None,
    RetryExitFullscreen,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingHide {
    None,
    WaitingForFullscreenExitUntil(Instant),
    WaitingUntil(Instant),
}

#[derive(Debug)]
struct MainWindowCloseState {
    pending_hide: PendingHide,
}

impl MainWindowCloseState {
    fn new() -> Self {
        Self {
            pending_hide: PendingHide::None,
        }
    }

    fn on_close_requested(
        &mut self,
        is_fullscreen: Option<bool>,
        now: Instant,
    ) -> CloseRequestAction {
        match is_fullscreen {
            Some(false) => {
                self.pending_hide = PendingHide::None;
                CloseRequestAction::HideImmediately
            }
            Some(true) | None => {
                self.pending_hide =
                    PendingHide::WaitingForFullscreenExitUntil(now + FULLSCREEN_EXIT_TIMEOUT);
                CloseRequestAction::ExitFullscreenThenHide
            }
        }
    }

    fn on_main_events_cleared(
        &mut self,
        is_fullscreen: Option<bool>,
        now: Instant,
    ) -> MainEventsAction {
        match self.pending_hide {
            PendingHide::None => MainEventsAction::None,
            PendingHide::WaitingForFullscreenExitUntil(deadline) => match is_fullscreen {
                Some(true) if now < deadline => MainEventsAction::None,
                Some(true) => {
                    self.pending_hide =
                        PendingHide::WaitingForFullscreenExitUntil(now + FULLSCREEN_EXIT_TIMEOUT);
                    MainEventsAction::RetryExitFullscreen
                }
                Some(false) | None => {
                    self.pending_hide = PendingHide::WaitingUntil(now + FULLSCREEN_HIDE_DELAY);
                    MainEventsAction::None
                }
            },
            PendingHide::WaitingUntil(deadline) if now < deadline => MainEventsAction::None,
            PendingHide::WaitingUntil(_) => {
                self.pending_hide = PendingHide::None;
                MainEventsAction::Hide
            }
        }
    }

    fn reset(&mut self) {
        self.pending_hide = PendingHide::None;
    }

    fn has_pending_hide(&self) -> bool {
        !matches!(self.pending_hide, PendingHide::None)
    }
}

static MAIN_WINDOW_CLOSE_STATE: LazyLock<Mutex<MainWindowCloseState>> =
    LazyLock::new(|| Mutex::new(MainWindowCloseState::new()));

fn read_fullscreen_state(result: tauri::Result<bool>, log_errors: bool) -> Option<bool> {
    match result {
        Ok(is_fullscreen) => Some(is_fullscreen),
        Err(error) => {
            if log_errors {
                eprintln!("查询 macOS 主窗口全屏状态失败: {error}");
            }
            None
        }
    }
}

fn reset_pending_hide() {
    if let Ok(mut state) = MAIN_WINDOW_CLOSE_STATE.lock() {
        state.reset();
    }
}

fn hide_main_window(app_handle: &tauri::AppHandle) {
    let app_handle_for_closure = app_handle.clone();

    if let Err(error) = app_handle.run_on_main_thread(move || {
        let mtm = MainThreadMarker::new().expect("window hide should run on main thread");
        let app = NSApp(mtm);

        if let Some(window) = app_handle_for_closure.get_webview_window(MAIN_WINDOW_LABEL) {
            if let Ok(ns_window) = window.ns_window() {
                let ns_window: &NSWindow = unsafe { &*ns_window.cast() };
                ns_window.orderOut(None);
            }
        }

        app.hide(None);
    }) {
        eprintln!("macOS 主线程隐藏窗口失败: {error}");
        reset_pending_hide();
    }
}

pub fn handle_close_requested(window: &tauri::Window, api: &tauri::CloseRequestApi) {
    let close_action = MAIN_WINDOW_CLOSE_STATE
        .lock()
        .map(|mut state| {
            state.on_close_requested(
                read_fullscreen_state(window.is_fullscreen(), true),
                Instant::now(),
            )
        })
        .unwrap_or(CloseRequestAction::HideImmediately);

    match close_action {
        CloseRequestAction::HideImmediately => {
            api.prevent_close();
            hide_main_window(window.app_handle());
        }
        CloseRequestAction::ExitFullscreenThenHide => {
            api.prevent_close();
            if let Err(error) = window.set_fullscreen(false) {
                eprintln!("macOS 关闭时退出全屏失败: {error}");
            }
        }
    }
}

pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.label() != MAIN_WINDOW_LABEL {
            return;
        }
        handle_close_requested(window, api);
    }
}

pub fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
    if let RunEvent::MainEventsCleared = event {
        let Some(window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL) else {
            return;
        };

        let now = Instant::now();
        let needs_fullscreen_state = MAIN_WINDOW_CLOSE_STATE
            .lock()
            .map(|state| state.has_pending_hide())
            .unwrap_or(false);
        let fullscreen_state = if needs_fullscreen_state {
            read_fullscreen_state(window.is_fullscreen(), false)
        } else {
            None
        };

        let action = MAIN_WINDOW_CLOSE_STATE
            .lock()
            .map(|mut state| state.on_main_events_cleared(fullscreen_state, now))
            .unwrap_or(MainEventsAction::None);

        match action {
            MainEventsAction::None => {}
            MainEventsAction::RetryExitFullscreen => {
                let _ = window.set_fullscreen(false);
            }
            MainEventsAction::Hide => {
                hide_main_window(app_handle);
            }
        }
    }
}
