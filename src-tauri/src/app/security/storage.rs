use super::model::SecuritySettings;
use crate::app::settings::storage;

const SECURITY_SETTINGS_FILE: &str = "security.json";
const CONTEXT: &str = "安全设置";

pub fn read_security_settings(app: &tauri::AppHandle) -> Result<SecuritySettings, String> {
    match storage::read_json_settings(app, SECURITY_SETTINGS_FILE, CONTEXT)? {
        Some(settings) => Ok(settings),
        None => {
            let settings = SecuritySettings::default();
            write_security_settings(app, &settings)?;
            Ok(settings)
        }
    }
}

pub fn write_security_settings(
    app: &tauri::AppHandle,
    settings: &SecuritySettings,
) -> Result<(), String> {
    storage::write_json_settings(app, SECURITY_SETTINGS_FILE, settings, CONTEXT)
}
