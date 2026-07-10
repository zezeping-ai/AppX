use std::fs;
use std::path::{Path, PathBuf};

use tauri::Manager;

use super::model::SecuritySettings;

const SECURITY_SETTINGS_FILE: &str = "security.json";

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位应用数据目录：{err}"))
}

fn security_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(SECURITY_SETTINGS_FILE))
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("读取安全设置失败：{err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("解析安全设置失败：{err}"))
}

pub fn read_security_settings(app: &tauri::AppHandle) -> Result<SecuritySettings, String> {
    let path = security_settings_path(app)?;
    if path.exists() {
        return read_json_file(&path);
    }

    let settings = SecuritySettings::default();
    write_security_settings(app, &settings)?;
    Ok(settings)
}

pub fn write_security_settings(
    app: &tauri::AppHandle,
    settings: &SecuritySettings,
) -> Result<(), String> {
    let path = security_settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建安全设置目录失败：{err}"))?;
    }

    let raw = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("序列化安全设置失败：{err}"))?;
    fs::write(path, raw).map_err(|err| format!("写入安全设置失败：{err}"))
}
