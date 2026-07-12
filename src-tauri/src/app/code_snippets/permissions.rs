use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};

use super::expansion;
use super::registry::SnippetRegistry;
use crate::app::palette::read_palette_settings;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippetPermissionsView {
    pub platform: &'static str,
    pub accessibility_granted: Option<bool>,
    /// 当前平台是否支持 `:缩写` + 触发键 全局展开
    pub abbreviation_supported: bool,
    pub enabled: bool,
    pub inline_expansion_enabled: bool,
    pub inline_expansion_trigger: String,
    pub shortcuts_enabled: bool,
    pub palette_enabled: bool,
    pub palette_shortcut: String,
    pub listener_active: bool,
    pub registered_abbreviation_count: usize,
    pub registered_abbreviations: Vec<String>,
}

#[tauri::command]
pub fn code_snippets_get_permissions(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<CodeSnippetPermissionsView, String> {
    ensure_unlocked(&state)?;
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

    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    Ok(CodeSnippetPermissionsView {
        platform,
        accessibility_granted: accessibility_granted(),
        abbreviation_supported: abbreviation_supported(),
        enabled: snippet_settings.enabled,
        inline_expansion_enabled: snippet_settings.inline_expansion_enabled,
        inline_expansion_trigger: snippet_settings.inline_expansion_trigger,
        shortcuts_enabled: snippet_settings.shortcuts_enabled,
        palette_enabled: snippet_settings.palette_enabled,
        palette_shortcut: palette_settings.palette_shortcut,
        listener_active,
        registered_abbreviation_count,
        registered_abbreviations,
    })
}

#[tauri::command]
pub fn code_snippets_open_accessibility_settings(
    state: State<'_, AppLockSessionState>,
) -> Result<(), String> {
    ensure_unlocked(&state)?;
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
        open_accessibility_settings_non_macos()
    }
}

fn abbreviation_supported() -> bool {
    #[cfg(target_os = "macos")]
    {
        return true;
    }
    #[cfg(target_os = "windows")]
    {
        return true;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return crate::app::platform::is_x11_session();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
fn open_accessibility_settings_non_macos() -> Result<(), String> {
    std::process::Command::new("ms-settings:privacy-accessibility")
        .spawn()
        .map_err(|err| format!("打开无障碍设置失败：{err}"))?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_accessibility_settings_non_macos() -> Result<(), String> {
    // 各桌面环境入口不同，优先尝试 GNOME 设置。
    let _ = std::process::Command::new("xdg-open")
        .arg("gnome-control-center universal-access")
        .spawn();
    Ok(())
}

#[cfg(target_os = "macos")]
fn accessibility_granted() -> Option<bool> {
    Some(is_accessibility_trusted())
}

#[cfg(target_os = "windows")]
fn accessibility_granted() -> Option<bool> {
    // Windows UIA 无统一查询 API，由用户自行确认。
    None
}

#[cfg(all(unix, not(target_os = "macos")))]
fn accessibility_granted() -> Option<bool> {
    std::env::var("AT_SPI_BUS_ADDRESS")
        .ok()
        .filter(|v| !v.is_empty())
        .map(|_| true)
}

#[cfg(target_os = "macos")]
fn is_accessibility_trusted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}
