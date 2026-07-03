use sha2::{Digest, Sha256};

use super::model::{EditorEncryptionView, EditorSettingsView};
use super::storage::{read_editor_settings, write_editor_settings};

pub const KEY_LEN: usize = 32;

pub fn derive_key_from_passphrase(passphrase: &str) -> [u8; KEY_LEN] {
    let digest = Sha256::digest(passphrase.as_bytes());
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&digest);
    key
}

pub fn load_passphrase(app: &tauri::AppHandle) -> Result<String, String> {
    Ok(read_editor_settings(app)?.encryption.passphrase)
}

pub fn load_cipher_key(app: &tauri::AppHandle) -> Result<[u8; KEY_LEN], String> {
    Ok(derive_key_from_passphrase(&load_passphrase(app)?))
}

pub fn editor_settings_view(app: &tauri::AppHandle) -> Result<EditorSettingsView, String> {
    let settings = read_editor_settings(app)?;
    Ok(EditorSettingsView {
        encryption: EditorEncryptionView {
            passphrase: settings.encryption.passphrase,
        },
    })
}

pub fn save_encryption_passphrase(
    app: &tauri::AppHandle,
    passphrase: &str,
) -> Result<EditorSettingsView, String> {
    let trimmed = passphrase.trim();
    if trimmed.is_empty() {
        return Err("加密口令不能为空".to_string());
    }

    let mut settings = read_editor_settings(app)?;
    settings.encryption.passphrase = trimmed.to_string();
    write_editor_settings(app, &settings)?;

    Ok(EditorSettingsView {
        encryption: EditorEncryptionView {
            passphrase: trimmed.to_string(),
        },
    })
}
