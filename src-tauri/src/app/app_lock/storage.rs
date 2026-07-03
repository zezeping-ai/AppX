use std::fs;
use std::path::{Path, PathBuf};

use tauri::Manager;

use super::model::AppLockSettings;

const APP_LOCK_SETTINGS_FILE: &str = "app-lock.json";

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位应用数据目录：{err}"))
}

fn app_lock_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(APP_LOCK_SETTINGS_FILE))
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("读取应用锁设置失败：{err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("解析应用锁设置失败：{err}"))
}

pub fn read_app_lock_settings(app: &tauri::AppHandle) -> Result<AppLockSettings, String> {
    let path = app_lock_settings_path(app)?;
    if path.exists() {
        return read_json_file(&path);
    }

    Ok(AppLockSettings::default())
}

pub fn write_app_lock_settings(
    app: &tauri::AppHandle,
    settings: &AppLockSettings,
) -> Result<(), String> {
    let path = app_lock_settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建应用锁目录失败：{err}"))?;
    }

    let raw = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("序列化应用锁设置失败：{err}"))?;
    fs::write(path, raw).map_err(|err| format!("写入应用锁设置失败：{err}"))
}
