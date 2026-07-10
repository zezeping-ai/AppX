//! 跨软件 Code Snippet 全局展开（`:abbreviation;` / 全局快捷键）

mod commands;
mod expansion;
mod model;
mod permissions;
mod registry;
mod settings;

pub use commands::*;
pub use model::PaletteItem;
pub use permissions::*;
pub use registry::SnippetRegistry;

use tauri::AppHandle;

pub fn setup(app: &AppHandle) -> Result<(), String> {
    settings::load_runtime_flags(app);
    expansion::start(app)?;
    refresh_runtime(app)
}

/// 同步代码段运行时：刷新展开快捷键与命令面板快捷键。
pub fn refresh_runtime(app: &AppHandle) -> Result<(), String> {
    expansion::refresh_shortcuts(app)?;
    crate::app::palette::register_shortcut(app)
}
