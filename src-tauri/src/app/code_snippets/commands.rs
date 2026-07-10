use base64::Engine;
use tauri::{AppHandle, State};

use crate::app::crypto::decrypt_bytes_with_passphrase;
use crate::app::security::load_default_passphrase;

use super::expansion;
use super::model::{SnippetEntry, SnippetSyncItem};
use super::registry::SnippetRegistry;
use super::settings::{
    read_code_snippet_settings, set_inline_expansion_enabled, write_code_snippet_settings,
    CodeSnippetSettingsView, SaveCodeSnippetSettingsInput,
};

#[tauri::command]
pub fn code_snippets_set_expansion_paused(paused: bool) {
    expansion::set_expansion_paused(paused);
}

#[tauri::command]
pub fn code_snippets_sync(
    app: AppHandle,
    snippets: Vec<SnippetSyncItem>,
    registry: State<'_, SnippetRegistry>,
) -> Result<(), String> {
    let passphrase = load_default_passphrase(&app)?;
    let mut entries = Vec::with_capacity(snippets.len());

    for item in snippets {
        let abbreviation = item.abbreviation.trim().to_lowercase();
        if abbreviation.is_empty() {
            continue;
        }

        let payload = base64::engine::general_purpose::STANDARD
            .decode(item.content.trim())
            .map_err(|err| format!("snippet #{} `{}` 密文无效：{err}", item.id, abbreviation))?;
        let plain = decrypt_bytes_with_passphrase(&passphrase, &payload)?;
        let content = String::from_utf8(plain)
            .map_err(|err| format!("snippet `{}` 解密结果无效：{err}", abbreviation))?;

        let shortcut = item
            .shortcut
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        entries.push(SnippetEntry {
            id: item.id,
            name: item.name.trim().to_string(),
            abbreviation,
            shortcut,
            content,
        });
    }

    registry.replace(entries);
    super::refresh_runtime(&app)?;
    Ok(())
}

#[tauri::command]
pub fn code_snippets_get_settings(app: AppHandle) -> Result<CodeSnippetSettingsView, String> {
    Ok(CodeSnippetSettingsView::from(&read_code_snippet_settings(&app)?))
}

#[tauri::command]
pub fn code_snippets_save_settings(
    app: AppHandle,
    input: SaveCodeSnippetSettingsInput,
) -> Result<CodeSnippetSettingsView, String> {
    let mut settings = read_code_snippet_settings(&app)?;
    settings.inline_expansion_enabled = input.inline_expansion_enabled;
    write_code_snippet_settings(&app, &settings)?;
    set_inline_expansion_enabled(settings.inline_expansion_enabled);
    super::refresh_runtime(&app)?;
    Ok(CodeSnippetSettingsView::from(&settings))
}
