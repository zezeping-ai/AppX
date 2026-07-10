//! 应用数据路径，按 debug / release 隔离。

use std::fs::create_dir_all;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

pub const PROFILE_DEBUG: &str = "debug";
pub const PROFILE_RELEASE: &str = "release";
const DB_FILE_NAME: &str = "appx.db";

pub fn profile_name() -> &'static str {
    if cfg!(debug_assertions) {
        PROFILE_DEBUG
    } else {
        PROFILE_RELEASE
    }
}

/// plugin-sql 连接串（相对 app_config_dir）。
pub fn sqlite_connection_string() -> &'static str {
    if cfg!(debug_assertions) {
        "sqlite:debug/appx.db"
    } else {
        "sqlite:release/appx.db"
    }
}

pub fn app_profile_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("无法定位应用配置目录：{err}"))?;
    Ok(base.join(profile_name()))
}

pub fn ensure_app_profile_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_profile_dir(app)?;
    create_dir_all(&dir).map_err(|err| format!("无法创建配置目录 {}：{err}", dir.display()))?;
    Ok(dir)
}

pub fn database_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_profile_dir(app)?.join(DB_FILE_NAME))
}

pub fn ensure_profile_scaffold(app: &AppHandle) -> Result<(), String> {
    ensure_app_profile_dir(app)?;
    Ok(())
}
