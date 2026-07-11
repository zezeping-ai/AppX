use tauri::AppHandle;

use crate::app::app_lock::is_session_locked;

use super::settings::is_features_enabled;

const LOCKED_MESSAGE: &str = "应用已锁定，请先解锁";
const DISABLED_MESSAGE: &str = "剪切助手已在偏好设置中停用";

/// 功能 IPC / 浮层使用前：须已解锁且总开关开启。
pub fn ensure_usable(app: &AppHandle) -> Result<(), String> {
    if is_session_locked(app) {
        return Err(LOCKED_MESSAGE.to_string());
    }
    if !is_features_enabled() {
        return Err(DISABLED_MESSAGE.to_string());
    }
    Ok(())
}
