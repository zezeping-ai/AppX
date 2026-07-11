use tauri::{AppHandle, Manager};

use super::state::AppLockSessionState;

const LOCKED_MESSAGE: &str = "应用已锁定，请先解锁";

pub fn ensure_unlocked(state: &AppLockSessionState) -> Result<(), String> {
    if state.is_locked()? {
        return Err(LOCKED_MESSAGE.to_string());
    }
    Ok(())
}

pub fn is_session_locked(app: &AppHandle) -> bool {
    app.try_state::<AppLockSessionState>()
        .and_then(|state| state.is_locked().ok())
        .unwrap_or(false)
}
