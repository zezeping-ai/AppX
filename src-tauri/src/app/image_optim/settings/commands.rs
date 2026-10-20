use tauri::{Emitter, State};

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};

use super::model::ImageOptimSettings;
use super::storage::{read_image_optim_settings, write_image_optim_settings};

pub const SETTINGS_CHANGED_EVENT: &str = "appx:image-optim-settings-changed";

#[tauri::command]
pub fn image_optim_get_settings(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<ImageOptimSettings, String> {
    ensure_unlocked(&state)?;
    read_image_optim_settings(&app)
}

#[tauri::command]
pub fn image_optim_save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    settings: ImageOptimSettings,
) -> Result<ImageOptimSettings, String> {
    ensure_unlocked(&state)?;
    let settings = settings.sanitized();
    write_image_optim_settings(&app, &settings)?;
    let _ = app.emit(SETTINGS_CHANGED_EVENT, &settings);
    Ok(settings)
}
