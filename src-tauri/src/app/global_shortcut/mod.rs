//! 全局快捷键协调：录制暂停 + 各域 provider 重注册，避免功能域互相调用。

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::app::app_lock::is_session_locked;
use crate::app::code_snippets;
use crate::app::runtime;

type ShortcutProvider = fn(&AppHandle) -> Result<(), String>;

static PAUSE_DEPTH: AtomicU32 = AtomicU32::new(0);
static PROVIDERS: Mutex<Vec<ShortcutProvider>> = Mutex::new(Vec::new());

#[tauri::command]
pub fn global_shortcuts_set_paused(app: AppHandle, paused: bool) -> Result<(), String> {
    set_paused(&app, paused)
}

/// 注册会话锁/解锁时的快捷键清理与恢复（应在各域 setup 之后调用一次）。
pub fn setup() {
    runtime::register_on_locked(on_locked);
    runtime::register_on_unlocked(on_unlocked);
}

fn on_locked(app: &AppHandle) -> Result<(), String> {
    clear_all(app)
}

fn on_unlocked(app: &AppHandle) -> Result<(), String> {
    refresh_all(app)
}

/// 各功能域在 setup 时注册；仅负责注册自己的快捷键，不得调用 unregister_all。
pub fn register_provider(provider: ShortcutProvider) {
    PROVIDERS.lock().unwrap().push(provider);
}

/// 清空全部全局快捷键后，按注册顺序让各域重注册。
pub fn refresh_all(app: &AppHandle) -> Result<(), String> {
    if PAUSE_DEPTH.load(Ordering::Relaxed) > 0 {
        return Ok(());
    }
    if is_session_locked(app) {
        return Ok(());
    }

    clear_all(app)?;

    for provider in PROVIDERS.lock().unwrap().iter() {
        provider(app)?;
    }
    Ok(())
}

pub fn clear_all(app: &AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|err| format!("取消注册全局快捷键失败：{err}"))
}

pub fn set_paused(app: &AppHandle, paused: bool) -> Result<(), String> {
    if paused {
        let depth = PAUSE_DEPTH.fetch_add(1, Ordering::Relaxed);
        if depth == 0 {
            clear_all(app)?;
            code_snippets::set_expansion_paused(true);
        }
        return Ok(());
    }

    // fetch_update 稳定；try_update 尚需 nightly
    let depth = PAUSE_DEPTH
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
        .unwrap_or(0);
    if depth != 1 {
        return Ok(());
    }

    if is_session_locked(app) {
        return Ok(());
    }

    code_snippets::set_expansion_paused(false);
    refresh_all(app)
}
