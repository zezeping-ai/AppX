use std::fs;
use std::path::{Path, PathBuf};

use tauri::Manager;

use super::model::CodeSnippetSettings;

const SETTINGS_FILE: &str = "code-snippets-settings.json";

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位应用数据目录：{err}"))
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(SETTINGS_FILE))
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("读取代码段设置失败：{err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("解析代码段设置失败：{err}"))
}

pub fn read_code_snippet_settings(app: &tauri::AppHandle) -> Result<CodeSnippetSettings, String> {
    let path = settings_path(app)?;
    if path.exists() {
        return read_json_file(&path);
    }
    Ok(CodeSnippetSettings::default())
}

pub fn write_code_snippet_settings(
    app: &tauri::AppHandle,
    settings: &CodeSnippetSettings,
) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建代码段设置目录失败：{err}"))?;
    }
    let raw = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("序列化代码段设置失败：{err}"))?;
    fs::write(path, raw).map_err(|err| format!("写入代码段设置失败：{err}"))
}
