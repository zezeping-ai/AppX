use tauri::State;

use super::model::{to_view, AppLockSettingsView, SaveAppLockSettingsInput};
use super::runtime::{on_session_locked, on_session_unlocked};
use super::state::AppLockSessionState;
use super::storage::{read_app_lock_settings, write_app_lock_settings};

fn read_view(
    app: &tauri::AppHandle,
    state: &AppLockSessionState,
) -> Result<AppLockSettingsView, String> {
    let settings = read_app_lock_settings(app)?;
    let session_locked = state.is_locked()?;
    Ok(to_view(&settings, session_locked))
}

#[tauri::command]
pub fn app_lock_get_settings(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<AppLockSettingsView, String> {
    read_view(&app, &state)
}

#[tauri::command]
pub fn app_lock_save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    input: SaveAppLockSettingsInput,
) -> Result<AppLockSettingsView, String> {
    let mut settings = read_app_lock_settings(&app)?;
    settings.enabled = input.enabled;
    settings.lock_on_startup = input.lock_on_startup;
    settings.lock_on_window_show = input.lock_on_window_show;

    if !settings.enabled {
        state.set_locked(false)?;
        on_session_unlocked(&app)?;
    }

    write_app_lock_settings(&app, &settings)?;
    read_view(&app, &state)
}

#[tauri::command]
pub fn app_lock_lock_session(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<AppLockSettingsView, String> {
    let settings = read_app_lock_settings(&app)?;
    let should_lock = settings.enabled;
    state.set_locked(should_lock)?;
    if should_lock {
        on_session_locked(&app)?;
    }
    Ok(to_view(&settings, should_lock))
}

#[tauri::command]
pub fn app_lock_unlock_session(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<AppLockSettingsView, String> {
    let settings = read_app_lock_settings(&app)?;
    state.set_locked(false)?;
    on_session_unlocked(&app)?;
    Ok(to_view(&settings, false))
}
