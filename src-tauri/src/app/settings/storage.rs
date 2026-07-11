//! 应用数据目录下的 JSON 设置读写（各域 storage 共用）

use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位应用数据目录：{err}"))
}

pub fn settings_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(filename))
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path, context: &str) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("读取{context}失败：{err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("解析{context}失败：{err}"))
}

pub fn read_json_settings<T: serde::de::DeserializeOwned>(
    app: &AppHandle,
    filename: &str,
    context: &str,
) -> Result<Option<T>, String> {
    let path = settings_path(app, filename)?;
    if !path.exists() {
        return Ok(None);
    }
    read_json_file(&path, context).map(Some)
}

pub fn write_json_settings<T: serde::Serialize>(
    app: &AppHandle,
    filename: &str,
    settings: &T,
    context: &str,
) -> Result<(), String> {
    let path = settings_path(app, filename)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建{context}目录失败：{err}"))?;
    }
    let raw = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("序列化{context}失败：{err}"))?;
    fs::write(path, raw).map_err(|err| format!("写入{context}失败：{err}"))
}
