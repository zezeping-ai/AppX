use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use crate::app::platform;

fn icons_dir<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("无法定位应用数据目录：{e}"))
        .map(|d| d.join("clipboard_app_icons"))
}

/// 将 app_id 映射为安全的缓存文件名（跨平台路径/特殊字符）。
pub fn cache_filename(app_id: &str) -> String {
    let digest = Sha256::digest(app_id.as_bytes());
    digest.iter().take(16).map(|b| format!("{b:02x}")).collect()
}

pub fn icon_path(icons: &Path, app_id: &str) -> PathBuf {
    icons.join(format!("{}.png", cache_filename(app_id)))
}

pub fn icon_url_for_bundle(bundle: &str) -> String {
    format!(
        "clipboard-app-icon://localhost/{}",
        encode_uri_component(bundle)
    )
}

pub fn thumb_url_for_id(id: i64) -> String {
    format!("clipboard-thumb://localhost/{id}")
}

fn encode_uri_component(raw: &str) -> String {
    raw.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

/// 确保来源 App 图标已缓存到磁盘。
pub fn ensure_cached<R: tauri::Runtime>(app: &AppHandle<R>, app_id: &str) -> Result<PathBuf, String> {
    let icons = icons_dir(app)?;
    fs::create_dir_all(&icons).map_err(|e| format!("创建图标目录失败：{e}"))?;
    let dest = icon_path(&icons, app_id);
    if dest.is_file() {
        return Ok(dest);
    }
    if let Some(bytes) = platform::fetch_icon_bytes(app_id)? {
        fs::write(&dest, &bytes).map_err(|e| format!("写入图标失败：{e}"))?;
        return Ok(dest);
    }
    Err(format!("无法获取 App 图标：{app_id}"))
}

pub fn read_cached<R: tauri::Runtime>(app: &AppHandle<R>, app_id: &str) -> Option<Vec<u8>> {
    let icons = icons_dir(app).ok()?;
    let path = icon_path(&icons, app_id);
    fs::read(path).ok()
}
