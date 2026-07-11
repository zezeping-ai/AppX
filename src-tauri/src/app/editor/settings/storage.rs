use std::fs;

use super::model::{EditorSettings, LegacyAppSettings};
use crate::app::settings::storage;

const EDITOR_SETTINGS_FILE: &str = "editor.json";
const LEGACY_SETTINGS_FILE: &str = "settings.json";
const CONTEXT: &str = "编辑器设置";

fn migrate_legacy_settings(app: &tauri::AppHandle) -> Result<Option<EditorSettings>, String> {
    let legacy_path = storage::settings_path(app, LEGACY_SETTINGS_FILE)?;
    if !legacy_path.exists() {
        return Ok(None);
    }

    let legacy = storage::read_json_settings::<LegacyAppSettings>(app, LEGACY_SETTINGS_FILE, CONTEXT)?
        .ok_or_else(|| "legacy settings file exists but could not be read".to_string())?;
    let settings = EditorSettings {
        encryption: super::model::EditorEncryptionSettings {
            passphrase: legacy.encryption_passphrase,
        },
    };
    write_editor_settings(app, &settings)?;
    let _ = fs::remove_file(&legacy_path);
    Ok(Some(settings))
}

pub fn read_editor_settings(app: &tauri::AppHandle) -> Result<EditorSettings, String> {
    if let Some(settings) = storage::read_json_settings(app, EDITOR_SETTINGS_FILE, CONTEXT)? {
        return Ok(settings);
    }

    if let Some(settings) = migrate_legacy_settings(app)? {
        return Ok(settings);
    }

    Ok(EditorSettings::default())
}

pub fn write_editor_settings(app: &tauri::AppHandle, settings: &EditorSettings) -> Result<(), String> {
    storage::write_json_settings(app, EDITOR_SETTINGS_FILE, settings, CONTEXT)
}
