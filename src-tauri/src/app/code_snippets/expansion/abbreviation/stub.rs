//! 非 macOS 平台暂未实现全局缩写监听。

use tauri::AppHandle;

pub fn start_listener(_app: AppHandle) -> Result<(), String> {
    log::warn!("[code_snippets] :abbreviation; expansion is only supported on macOS");
    Ok(())
}
