use tauri::State;

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};

use super::key;
use super::model::EditorSettingsView;

#[tauri::command]
pub fn editor_get_settings(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<EditorSettingsView, String> {
    ensure_unlocked(&state)?;
    key::editor_settings_view(&app)
}

#[tauri::command]
pub fn editor_save_encryption_passphrase(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    passphrase: String,
) -> Result<EditorSettingsView, String> {
    ensure_unlocked(&state)?;
    key::save_encryption_passphrase(&app, &passphrase)
}
