//! 向 runtime 协调器注册会话锁/解锁回调，避免 app_lock 直接依赖本模块

use tauri::AppHandle;

use crate::app::code_snippets;
use crate::app::palette;
use crate::app::runtime;

pub fn register_hooks() {
    runtime::register_on_locked(on_locked);
    runtime::register_on_unlocked(on_unlocked);
}

fn on_locked(app: &AppHandle) -> Result<(), String> {
    code_snippets::set_expansion_paused(true);
    // 全局快捷键清空由 global_shortcut::setup 的 on_locked 统一处理
    palette::dismiss_palette_window(app)
}

fn on_unlocked(_app: &AppHandle) -> Result<(), String> {
    // 快捷键由 global_shortcut::setup 在解锁链末尾统一 refresh_all
    code_snippets::set_expansion_paused(false);
    Ok(())
}
