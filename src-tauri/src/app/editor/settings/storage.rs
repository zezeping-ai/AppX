use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

use super::model::{EditorSettings, LegacyAppSettings};

const EDITOR_SETTINGS_FILE: &str = "editor.json";
const LEGACY_SETTINGS_FILE: &str = "settings.json";

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位应用数据目录：{err}"))
}

fn editor_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(EDITOR_SETTINGS_FILE))
}

fn legacy_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(LEGACY_SETTINGS_FILE))
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("读取设置失败：{err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("解析设置失败：{err}"))
}

fn migrate_legacy_settings(app: &tauri::AppHandle) -> Result<Option<EditorSettings>, String> {
    let legacy_path = legacy_settings_path(app)?;
    if !legacy_path.exists() {
        return Ok(None);
    }

    let legacy = read_json_file::<LegacyAppSettings>(&legacy_path)?;
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
    let path = editor_settings_path(app)?;
    if path.exists() {
        return read_json_file(&path);
    }

    if let Some(settings) = migrate_legacy_settings(app)? {
        return Ok(settings);
    }

    Ok(EditorSettings::default())
}

pub fn write_editor_settings(app: &tauri::AppHandle, settings: &EditorSettings) -> Result<(), String> {
    let path = editor_settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建设置目录失败：{err}"))?;
    }

    let raw = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("序列化设置失败：{err}"))?;
    fs::write(path, raw).map_err(|err| format!("写入设置失败：{err}"))
}
