//! 非支持平台（如 Linux Wayland）的缩写监听占位。

use tauri::AppHandle;

pub fn start_listener(_app: AppHandle) -> Result<(), String> {
    log::warn!("[code_snippets] :abbrev expansion requires macOS, Windows, or Linux X11");
    Ok(())
}
