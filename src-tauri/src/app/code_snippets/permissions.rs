use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::expansion;
use super::registry::SnippetRegistry;
use crate::app::palette::read_palette_settings;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippetPermissionsView {
    pub platform: &'static str,
    pub accessibility_granted: Option<bool>,
    /// 当前平台是否支持 `:缩写;` 全局展开
    pub abbreviation_supported: bool,
    pub inline_expansion_enabled: bool,
    pub palette_shortcut: String,
    pub listener_active: bool,
    pub registered_abbreviation_count: usize,
    pub registered_abbreviations: Vec<String>,
}

#[tauri::command]
pub fn code_snippets_get_permissions(app: AppHandle) -> CodeSnippetPermissionsView {
    let (registered_abbreviation_count, registered_abbreviations) = app
        .try_state::<SnippetRegistry>()
        .map(|registry| {
            let snapshot = registry.snapshot();
            let mut abbrevs: Vec<String> = snapshot.by_abbreviation.keys().cloned().collect();
            abbrevs.sort();
            (abbrevs.len(), abbrevs)
        })
        .unwrap_or((0, Vec::new()));

    let listener_active = expansion::is_abbreviation_listener_active();
    let snippet_settings = super::settings::read_code_snippet_settings(&app).unwrap_or_default();
    let palette_settings = read_palette_settings(&app).unwrap_or_default();

    #[cfg(target_os = "macos")]
    {
        CodeSnippetPermissionsView {
            platform: "macos",
            accessibility_granted: Some(is_accessibility_trusted()),
            abbreviation_supported: true,
            inline_expansion_enabled: snippet_settings.inline_expansion_enabled,
            palette_shortcut: palette_settings.palette_shortcut,
            listener_active,
            registered_abbreviation_count,
            registered_abbreviations,
        }
    }
    #[cfg(target_os = "windows")]
    {
        CodeSnippetPermissionsView {
            platform: "windows",
            accessibility_granted: None,
            abbreviation_supported: false,
            inline_expansion_enabled: snippet_settings.inline_expansion_enabled,
            palette_shortcut: palette_settings.palette_shortcut,
            listener_active,
            registered_abbreviation_count,
            registered_abbreviations,
        }
    }
    #[cfg(target_os = "linux")]
    {
        CodeSnippetPermissionsView {
            platform: "linux",
            accessibility_granted: None,
            abbreviation_supported: false,
            inline_expansion_enabled: snippet_settings.inline_expansion_enabled,
            palette_shortcut: palette_settings.palette_shortcut,
            listener_active,
            registered_abbreviation_count,
            registered_abbreviations,
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        CodeSnippetPermissionsView {
            platform: "unknown",
            accessibility_granted: None,
            abbreviation_supported: false,
            inline_expansion_enabled: snippet_settings.inline_expansion_enabled,
            palette_shortcut: palette_settings.palette_shortcut,
            listener_active,
            registered_abbreviation_count,
            registered_abbreviations,
        }
    }
}

#[tauri::command]
pub fn code_snippets_open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        const URL: &str =
            "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Accessibility";
        std::process::Command::new("open")
            .arg(URL)
            .spawn()
            .map_err(|err| format!("打开系统设置失败：{err}"))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("仅 macOS 支持打开辅助功能设置".to_string())
    }
}

#[cfg(target_os = "macos")]
fn is_accessibility_trusted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}
