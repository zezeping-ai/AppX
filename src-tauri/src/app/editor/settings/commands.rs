use super::key;
use super::model::EditorSettingsView;

#[tauri::command]
pub fn editor_get_settings(app: tauri::AppHandle) -> Result<EditorSettingsView, String> {
    key::editor_settings_view(&app)
}

#[tauri::command]
pub fn editor_save_encryption_passphrase(
    app: tauri::AppHandle,
    passphrase: String,
) -> Result<EditorSettingsView, String> {
    key::save_encryption_passphrase(&app, &passphrase)
}
