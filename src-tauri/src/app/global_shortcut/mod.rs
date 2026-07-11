//! 全局快捷键录制期间暂停注册，避免与 ShortcutRecorder 冲突。

use std::sync::atomic::{AtomicU32, Ordering};

use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::app::app_lock::is_session_locked;
use crate::app::code_snippets;

static PAUSE_DEPTH: AtomicU32 = AtomicU32::new(0);

#[tauri::command]
pub fn global_shortcuts_set_paused(app: AppHandle, paused: bool) -> Result<(), String> {
    set_paused(&app, paused)
}

pub fn set_paused(app: &AppHandle, paused: bool) -> Result<(), String> {
    if paused {
        let depth = PAUSE_DEPTH.fetch_add(1, Ordering::Relaxed);
        if depth == 0 {
            app.global_shortcut()
                .unregister_all()
                .map_err(|err| format!("取消注册全局快捷键失败：{err}"))?;
            code_snippets::set_expansion_paused(true);
        }
        return Ok(());
    }

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
    code_snippets::refresh_runtime(app)
}
