use crate::app::runtime;

/// 锁定会话时通知各功能域暂停全局能力。
pub fn on_session_locked(app: &tauri::AppHandle) -> Result<(), String> {
    runtime::on_session_locked(app)
}

/// 解锁会话后通知各功能域恢复运行时。
pub fn on_session_unlocked(app: &tauri::AppHandle) -> Result<(), String> {
    runtime::on_session_unlocked(app)
}
