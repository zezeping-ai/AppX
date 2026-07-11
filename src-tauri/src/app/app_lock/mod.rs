mod commands;
mod guard;
mod model;
mod runtime;
mod state;
mod storage;

use tauri::Manager;

pub use commands::*;
pub use guard::{ensure_unlocked, is_session_locked};
pub use runtime::on_session_locked;
pub use state::AppLockSessionState;

pub fn setup(
    app: &tauri::AppHandle,
    state: tauri::State<'_, AppLockSessionState>,
) -> Result<(), String> {
    let settings = storage::read_app_lock_settings(app)?;
    let should_lock = settings.enabled && settings.lock_on_startup;
    state.set_locked(should_lock)?;
    if should_lock {
        on_session_locked(app)?;
    }
    Ok(())
}

pub fn relock_on_show(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = storage::read_app_lock_settings(app)?;
    let should_lock = settings.enabled && settings.lock_on_window_show;
    let state = app.state::<AppLockSessionState>();
    state.set_locked(should_lock)?;
    if should_lock {
        on_session_locked(app)?;
    }
    Ok(())
}
