use super::model::CodeSnippetSettings;
use crate::app::settings::storage;

const SETTINGS_FILE: &str = "code-snippets-settings.json";
const CONTEXT: &str = "代码段设置";

pub fn read_code_snippet_settings(app: &tauri::AppHandle) -> Result<CodeSnippetSettings, String> {
    match storage::read_json_settings(app, SETTINGS_FILE, CONTEXT)? {
        Some(settings) => Ok(settings),
        None => Ok(CodeSnippetSettings::default()),
    }
}

pub fn write_code_snippet_settings(
    app: &tauri::AppHandle,
    settings: &CodeSnippetSettings,
) -> Result<(), String> {
    storage::write_json_settings(app, SETTINGS_FILE, settings, CONTEXT)
}
