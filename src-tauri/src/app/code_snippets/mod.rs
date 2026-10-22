//! 跨软件 Code Snippet 全局展开（`:abbrev` + F12 / 全局快捷键）

mod commands;
mod db;
mod expansion;
mod model;
mod permissions;
mod resolve;
mod registry;
mod session;
mod settings;

pub use commands::*;
pub use db::SnippetDb;
pub use model::PaletteItem;
pub use permissions::*;
pub use registry::SnippetRegistry;
pub use settings::{is_palette_enabled, is_shortcuts_enabled};

use tauri::{AppHandle, Manager};

use crate::app::app_lock::is_session_locked;
use crate::app::global_shortcut;
use crate::app::palette;

/// 触发时解析粘贴明文（查库 + 按 app 选变体 + 解密；命中缓存则跳过 Argon2）。
/// `target_app_id` 为空则用当前前台。
pub(crate) fn resolve_plaintext(
    app: &AppHandle,
    id: i64,
    target_app_id: Option<&str>,
) -> Result<String, String> {
    match target_app_id {
        Some(app_id) => resolve::resolve_plain_content_for_app(app, id, Some(app_id)),
        None => resolve::resolve_plain_content(app, id),
    }
}

/// 会话锁定时清空明文缓存与复用连接。
pub fn clear_runtime_caches(app: &AppHandle) {
    if let Some(registry) = app.try_state::<SnippetRegistry>() {
        registry.clear_plaintext_cache();
    }
    if let Some(db) = app.try_state::<SnippetDb>() {
        db.reset();
    }
}

pub fn set_expansion_paused(paused: bool) {
    expansion::set_expansion_paused(paused);
}

pub fn setup(app: &AppHandle) -> Result<(), String> {
    session::register_hooks();
    settings::load_runtime_flags(app);
    global_shortcut::register_provider(register_own_shortcuts);
    expansion::start(app)?;
    refresh_runtime(app)
}

/// 同步代码段运行时：经协调器重注册全部全局快捷键（含其他域 provider）。
pub fn refresh_runtime(app: &AppHandle) -> Result<(), String> {
    if is_session_locked(app) {
        expansion::set_expansion_paused(true);
        return Ok(());
    }

    expansion::set_expansion_paused(false);
    global_shortcut::refresh_all(app)
}

/// 仅注册本域快捷键（snippet 展开键 + 命令面板）。
fn register_own_shortcuts(app: &AppHandle) -> Result<(), String> {
    expansion::refresh_shortcuts(app)?;
    palette::register_shortcut(app)
}
