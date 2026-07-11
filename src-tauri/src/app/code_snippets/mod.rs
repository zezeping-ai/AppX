//! 跨软件 Code Snippet 全局展开（`:abbrev` + F12 / 全局快捷键）

mod commands;
mod expansion;
mod model;
mod permissions;
mod registry;
mod session;
mod settings;

pub use commands::*;
pub use model::PaletteItem;
pub use permissions::*;
pub use registry::SnippetRegistry;
pub use settings::{is_palette_enabled, is_shortcuts_enabled};

use tauri::AppHandle;

use crate::app::app_lock::is_session_locked;

pub fn set_expansion_paused(paused: bool) {
    expansion::set_expansion_paused(paused);
}

pub fn setup(app: &AppHandle) -> Result<(), String> {
    session::register_hooks();
    settings::load_runtime_flags(app);
    expansion::start(app)?;
    refresh_runtime(app)
}

/// 同步代码段运行时：刷新展开快捷键与命令面板快捷键。
pub fn refresh_runtime(app: &AppHandle) -> Result<(), String> {
    if is_session_locked(app) {
        expansion::set_expansion_paused(true);
        return Ok(());
    }

    expansion::set_expansion_paused(false);
    expansion::refresh_shortcuts(app)?;
    crate::app::palette::register_shortcut(app)
}
