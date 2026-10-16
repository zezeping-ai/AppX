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
use crate::app::global_shortcut;
use crate::app::palette;

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
