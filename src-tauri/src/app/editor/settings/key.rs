use super::model::{EditorEncryptionView, EditorSettingsView};
use super::storage::{read_editor_settings, write_editor_settings};
use crate::app::security::load_default_passphrase;

/// 解析 Editor 实际使用的默认加密口令：独立配置优先，未配置则回退「偏好设置 › 安全」
pub fn load_passphrase(app: &tauri::AppHandle) -> Result<String, String> {
    let settings = read_editor_settings(app)?;
    let editor_passphrase = settings.encryption.passphrase.trim();
    if !editor_passphrase.is_empty() {
        return Ok(editor_passphrase.to_string());
    }
    load_default_passphrase(app)
}

pub fn editor_settings_view(app: &tauri::AppHandle) -> Result<EditorSettingsView, String> {
    let settings = read_editor_settings(app)?;
    let uses_global = settings.encryption.passphrase.trim().is_empty();
    Ok(EditorSettingsView {
        encryption: EditorEncryptionView {
            passphrase: settings.encryption.passphrase,
            uses_global_passphrase: uses_global,
        },
    })
}

pub fn save_encryption_passphrase(
    app: &tauri::AppHandle,
    passphrase: &str,
) -> Result<EditorSettingsView, String> {
    let trimmed = passphrase.trim();

    let mut settings = read_editor_settings(app)?;
    settings.encryption.passphrase = trimmed.to_string();
    write_editor_settings(app, &settings)?;

    Ok(EditorSettingsView {
        encryption: EditorEncryptionView {
            passphrase: trimmed.to_string(),
            uses_global_passphrase: trimmed.is_empty(),
        },
    })
}
