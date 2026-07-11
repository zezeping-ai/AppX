use tauri::State;

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};

use super::model::SecuritySettingsView;
use super::passphrase::security_settings_view;

#[tauri::command]
pub fn security_get_settings(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<SecuritySettingsView, String> {
    ensure_unlocked(&state)?;
    security_settings_view(&app)
}
