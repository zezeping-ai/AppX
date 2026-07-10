use super::model::SecuritySettingsView;
use super::passphrase::security_settings_view;

#[tauri::command]
pub fn security_get_settings(app: tauri::AppHandle) -> Result<SecuritySettingsView, String> {
    security_settings_view(&app)
}
