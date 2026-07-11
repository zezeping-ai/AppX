use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::ImageFormat;
use tauri::{AppHandle, Manager};

fn icons_dir<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("无法定位应用数据目录：{e}"))
        .map(|d| d.join("clipboard_app_icons"))
}

pub fn icon_path(icons: &Path, bundle: &str) -> PathBuf {
    let safe = bundle.replace('.', "_");
    icons.join(format!("{safe}.png"))
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

/// 确保来源 App 图标已缓存到磁盘（macOS）。
pub fn ensure_cached<R: tauri::Runtime>(app: &AppHandle<R>, bundle: &str) -> Result<PathBuf, String> {
    let icons = icons_dir(app)?;
    fs::create_dir_all(&icons).map_err(|e| format!("创建图标目录失败：{e}"))?;
    let dest = icon_path(&icons, bundle);
    if dest.is_file() {
        return Ok(dest);
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(bytes) = fetch_icon_bytes_macos(bundle)? {
            fs::write(&dest, &bytes).map_err(|e| format!("写入图标失败：{e}"))?;
            return Ok(dest);
        }
    }
    Err(format!("无法获取 App 图标：{bundle}"))
}

pub fn read_cached<R: tauri::Runtime>(app: &AppHandle<R>, bundle: &str) -> Option<Vec<u8>> {
    let icons = icons_dir(app).ok()?;
    let path = icon_path(&icons, bundle);
    fs::read(path).ok()
}

#[cfg(target_os = "macos")]
fn fetch_icon_bytes_macos(bundle: &str) -> Result<Option<Vec<u8>>, String> {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::NSString;

    let workspace = NSWorkspace::sharedWorkspace();
    let ns_bundle = NSString::from_str(bundle);
    let Some(app_url) = workspace.URLForApplicationWithBundleIdentifier(&ns_bundle) else {
        return Ok(None);
    };
    let Some(app_path) = app_url.path() else {
        return Ok(None);
    };
    let icon = workspace.iconForFile(&app_path);
    let Some(tiff) = icon.TIFFRepresentation() else {
        return Ok(None);
    };
    let bytes: Vec<u8> = tiff.to_vec();
    let img = image::load_from_memory(&bytes).map_err(|e| format!("解码图标失败：{e}"))?;
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| format!("编码 PNG 失败：{e}"))?;
    Ok(Some(buf))
}

#[cfg(not(target_os = "macos"))]
fn fetch_icon_bytes_macos(_bundle: &str) -> Result<Option<Vec<u8>>, String> {
    Ok(None)
}
