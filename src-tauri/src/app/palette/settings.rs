use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::app::settings::storage;

const SETTINGS_FILE: &str = "palette-settings.json";
const LEGACY_SNIPPETS_SETTINGS: &str = "code-snippets-settings.json";
const CONTEXT: &str = "命令面板设置";

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

fn migrate_from_legacy(app: &AppHandle) -> Result<Option<PaletteSettings>, String> {
    let legacy_path = storage::settings_path(app, LEGACY_SNIPPETS_SETTINGS)?;
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

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = std::fs::read_to_string(path).map_err(|err| format!("读取{CONTEXT}失败：{err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("解析{CONTEXT}失败：{err}"))
}

fn write_settings(app: &AppHandle, settings: &PaletteSettings) -> Result<(), String> {
    storage::write_json_settings(app, SETTINGS_FILE, settings, CONTEXT)
}

pub fn read_palette_settings(app: &AppHandle) -> Result<PaletteSettings, String> {
    if let Some(settings) = storage::read_json_settings(app, SETTINGS_FILE, CONTEXT)? {
        return Ok(settings);
    }

    if let Some(settings) = migrate_from_legacy(app)? {
        return Ok(settings);
    }

    Ok(PaletteSettings::default())
}
