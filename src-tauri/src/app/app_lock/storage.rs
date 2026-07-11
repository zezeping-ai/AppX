use super::model::AppLockSettings;
use crate::app::settings::storage;

const APP_LOCK_SETTINGS_FILE: &str = "app-lock.json";
const CONTEXT: &str = "应用锁设置";

pub fn read_app_lock_settings(app: &tauri::AppHandle) -> Result<AppLockSettings, String> {
    match storage::read_json_settings(app, APP_LOCK_SETTINGS_FILE, CONTEXT)? {
        Some(settings) => Ok(settings),
        None => Ok(AppLockSettings::default()),
    }
}

pub fn write_app_lock_settings(
    app: &tauri::AppHandle,
    settings: &AppLockSettings,
) -> Result<(), String> {
    storage::write_json_settings(app, APP_LOCK_SETTINGS_FILE, settings, CONTEXT)
}
