use tauri::State;

use super::model::{
    to_view, AppLockSettingsView, SaveAppLockSettingsInput, UnlockSessionInput,
};
use super::pin::{hash_pin, verify_pin};
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

    if let Some(pin) = input.unlock_pin.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        settings.unlock_pin_hash = Some(hash_pin(pin)?);
    }

    if settings.enabled {
        let has_pin = settings
            .unlock_pin_hash
            .as_ref()
            .map(|h| !h.is_empty())
            .unwrap_or(false);
        if !has_pin {
            return Err("启用应用锁前请设置解锁 PIN".to_string());
        }
    }

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
    input: UnlockSessionInput,
) -> Result<AppLockSettingsView, String> {
    let settings = read_app_lock_settings(&app)?;
    match settings.unlock_pin_hash.as_ref().filter(|h| !h.is_empty()) {
        Some(hash) => verify_pin(&input.pin, hash)?,
        // 旧配置未设 PIN：允许解锁，但提示前端尽快设置
        None => {
            if !input.pin.trim().is_empty() {
                return Err("尚未设置解锁 PIN，请先在偏好设置中配置".to_string());
            }
        }
    }
    state.set_locked(false)?;
    on_session_unlocked(&app)?;
    Ok(to_view(&settings, false))
}
