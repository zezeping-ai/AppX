use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "palette-settings.json";
const LEGACY_SNIPPETS_SETTINGS: &str = "code-snippets-settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteSettings {
    #[serde(default = "default_palette_shortcut")]
    pub palette_shortcut: String,
}

fn default_palette_shortcut() -> String {
    "CommandOrControl+Shift+Space".to_string()
}

impl Default for PaletteSettings {
    fn default() -> Self {
        Self {
            palette_shortcut: default_palette_shortcut(),
        }
    }
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| format!("无法定位应用数据目录：{err}"))
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(SETTINGS_FILE))
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("读取命令面板设置失败：{err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("解析命令面板设置失败：{err}"))
}

fn write_settings(app: &AppHandle, settings: &PaletteSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建命令面板设置目录失败：{err}"))?;
    }
    let raw = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("序列化命令面板设置失败：{err}"))?;
    fs::write(path, raw).map_err(|err| format!("写入命令面板设置失败：{err}"))
}

fn migrate_from_legacy(app: &AppHandle) -> Result<Option<PaletteSettings>, String> {
    let legacy_path = app_data_dir(app)?.join(LEGACY_SNIPPETS_SETTINGS);
    if !legacy_path.exists() {
        return Ok(None);
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct LegacySnippetSettings {
        #[serde(default = "default_palette_shortcut")]
        palette_shortcut: String,
    }

    let legacy = read_json_file::<LegacySnippetSettings>(&legacy_path)?;
    let settings = PaletteSettings {
        palette_shortcut: legacy.palette_shortcut,
    };
    write_settings(app, &settings)?;
    Ok(Some(settings))
}

pub fn read_palette_settings(app: &AppHandle) -> Result<PaletteSettings, String> {
    let path = settings_path(app)?;
    if path.exists() {
        return read_json_file(&path);
    }

    if let Some(settings) = migrate_from_legacy(app)? {
        return Ok(settings);
    }

    Ok(PaletteSettings::default())
}
