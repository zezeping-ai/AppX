//! 向 runtime 协调器注册会话锁/解锁回调，避免 app_lock 直接依赖本模块

use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::app::code_snippets;
use crate::app::palette;
use crate::app::runtime;

pub fn register_hooks() {
    runtime::register_on_locked(on_locked);
    runtime::register_on_unlocked(on_unlocked);
}

fn on_locked(app: &AppHandle) -> Result<(), String> {
    code_snippets::set_expansion_paused(true);
    app.global_shortcut()
        .unregister_all()
        .map_err(|err| format!("取消注册全局快捷键失败：{err}"))?;
    palette::hide_palette_window(app)
}

fn on_unlocked(app: &AppHandle) -> Result<(), String> {
    code_snippets::set_expansion_paused(false);
    code_snippets::refresh_runtime(app)
}
