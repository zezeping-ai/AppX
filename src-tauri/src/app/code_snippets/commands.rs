use base64::Engine;
use tauri::{AppHandle, State};

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};
use crate::app::crypto::decrypt_bytes_with_passphrase;
use crate::app::security::load_default_passphrase;

use super::expansion;
use super::model::{SnippetEntry, SnippetSyncItem};
use super::registry::SnippetRegistry;
use super::settings::{
    apply_runtime_flags, read_code_snippet_settings, write_code_snippet_settings,
    CodeSnippetSettingsView, SaveCodeSnippetSettingsInput,
};

#[tauri::command]
pub fn code_snippets_set_expansion_paused(paused: bool) {
    expansion::set_expansion_paused(paused);
}

#[tauri::command]
pub fn code_snippets_sync(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
    snippets: Vec<SnippetSyncItem>,
    registry: State<'_, SnippetRegistry>,
) -> Result<(), String> {
    ensure_unlocked(&state)?;
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
            group: normalize_snippet_group(&item.group),
        });
    }

    registry.replace(entries);
    super::refresh_runtime(&app)?;
    Ok(())
}

#[tauri::command]
pub fn code_snippets_get_settings(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<CodeSnippetSettingsView, String> {
    ensure_unlocked(&state)?;
    Ok(CodeSnippetSettingsView::from(&read_code_snippet_settings(&app)?))
}

#[tauri::command]
pub fn code_snippets_save_settings(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
    input: SaveCodeSnippetSettingsInput,
) -> Result<CodeSnippetSettingsView, String> {
    ensure_unlocked(&state)?;
    let mut settings = read_code_snippet_settings(&app)?;
    settings.enabled = input.enabled;
    settings.inline_expansion_enabled = input.inline_expansion_enabled;
    settings.inline_expansion_trigger = normalize_trigger_shortcut(&input.inline_expansion_trigger);
    settings.shortcuts_enabled = input.shortcuts_enabled;
    settings.palette_enabled = input.palette_enabled;
    write_code_snippet_settings(&app, &settings)?;
    apply_runtime_flags(&settings);
    super::refresh_runtime(&app)?;
    Ok(CodeSnippetSettingsView::from(&settings))
}

fn normalize_snippet_group(raw: &str) -> String {
    let group = raw.trim();
    if group.is_empty() {
        "general".to_string()
    } else {
        group.to_string()
    }
}

fn normalize_trigger_shortcut(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "F12".to_string();
    }
    if tauri_plugin_global_shortcut::Shortcut::try_from(trimmed).is_ok() {
        trimmed.to_string()
    } else {
        "F12".to_string()
    }
}
