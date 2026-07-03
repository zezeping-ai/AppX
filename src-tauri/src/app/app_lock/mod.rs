mod commands;
mod model;
mod state;
mod storage;

use tauri::Manager;

pub use commands::*;
pub use state::AppLockSessionState;

pub fn setup(
    app: &tauri::AppHandle,
    state: tauri::State<'_, AppLockSessionState>,
) -> Result<(), String> {
    let settings = storage::read_app_lock_settings(app)?;
    let should_lock = settings.enabled && settings.lock_on_startup;
    state.set_locked(should_lock)
}

pub fn relock_on_show(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = storage::read_app_lock_settings(app)?;
    let should_lock = settings.enabled && settings.lock_on_startup;
    let state = app.state::<AppLockSessionState>();
    state.set_locked(should_lock)
}
