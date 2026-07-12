//! `:缩写` + 可配置触发键 全局缩写展开。
//! macOS：CGEventTap；Windows / Linux X11：global-shortcut + 无障碍 API 读光标前文本。

mod context;

#[cfg(target_os = "macos")]
mod foreground;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
mod global_shortcut_listener;

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    all(unix, not(target_os = "macos"))
)))]
mod stub;

pub mod trigger;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

use crate::app::app_lock::is_session_locked;
use crate::app::code_snippets::registry::SnippetRegistry;
use crate::app::code_snippets::settings::is_inline_expansion_enabled;
use crate::app::text_delivery::replace_trigger;

const MAX_ABBREV_LEN: usize = 32;
/// 回溯窗口 = `:` + 缩写，多留 1 字符容差
const LOOKBACK_LEN: usize = MAX_ABBREV_LEN + 1;

static EXPANSION_PAUSED: AtomicBool = AtomicBool::new(false);
static LISTENER_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn is_listener_active() -> bool {
    LISTENER_ACTIVE.load(Ordering::Relaxed)
}

pub fn set_listener_active(active: bool) {
    LISTENER_ACTIVE.store(active, Ordering::Relaxed);
}

pub fn set_expansion_paused(paused: bool) {
    EXPANSION_PAUSED.store(paused, Ordering::Relaxed);
}

pub fn is_expansion_paused() -> bool {
    EXPANSION_PAUSED.load(Ordering::Relaxed)
}

/// 触发键按下时从光标前文本回溯解析 `:缩写`，命中 registry 则展开。
pub(crate) fn try_expand_on_trigger(app: &AppHandle) {
    if is_session_locked(app) || is_expansion_paused() || !is_inline_expansion_enabled() {
        return;
    }

    let Some(text_before) = context::read_text_before_cursor(LOOKBACK_LEN) else {
        return;
    };

    let Some(abbrev) = parse_abbrev_from_context(&text_before) else {
        return;
    };

    let Some(registry) = app.try_state::<SnippetRegistry>() else {
        log::warn!("[code_snippets] registry unavailable for `{abbrev}`");
        return;
    };

    let key = abbrev.to_lowercase();
    let snapshot = registry.snapshot();
    let Some(entry) = snapshot.by_abbreviation.get(&key) else {
        log::debug!(
            "[code_snippets] no snippet for `{key}` (registered: {:?})",
            snapshot.by_abbreviation.keys().collect::<Vec<_>>()
        );
        return;
    };

    log::info!("[code_snippets] expanding `{key}` via lookback");
    replace_trigger(abbrev.chars().count(), &entry.content);
}

/// 在光标前文本窗口内找最近的 `:`，提取合法缩写。
pub(crate) fn parse_abbrev_from_context(text_before_cursor: &str) -> Option<String> {
    let window = take_suffix_chars(text_before_cursor, LOOKBACK_LEN);
    let colon = window.rfind(':')?;
    let abbrev = &window[colon + ':'.len_utf8()..];
    if abbrev.is_empty() || !is_valid_abbrev(abbrev) {
        return None;
    }
    Some(abbrev.to_string())
}

fn take_suffix_chars(text: &str, max_len: usize) -> &str {
    if text.chars().count() <= max_len {
        return text;
    }
    let skip = text.chars().count() - max_len;
    text.char_indices()
        .nth(skip)
        .map(|(idx, _)| &text[idx..])
        .unwrap_or(text)
}

fn is_valid_abbrev(abbrev: &str) -> bool {
    if abbrev.chars().count() > MAX_ABBREV_LEN {
        return false;
    }
    abbrev.chars().all(is_allowed_abbrev_char)
}

fn is_allowed_abbrev_char(ch: char) -> bool {
    if ch == ':' {
        return false;
    }
    ch.is_ascii_lowercase()
        || ch.is_ascii_digit()
        || matches!(
            ch,
            '-' | '_' | '!' | '#' | '%' | '^' | '&' | '*' | '(' | ')' | '+' | '=' | '.'
                | ',' | '?' | '/' | '<' | '>' | ';'
        )
}

#[cfg(target_os = "macos")]
pub fn start_listener(app: AppHandle) -> Result<(), String> {
    macos::start_listener(app)
}

#[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
pub fn start_listener(app: AppHandle) -> Result<(), String> {
    global_shortcut_listener::start_listener(app)
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    all(unix, not(target_os = "macos"))
)))]
pub fn start_listener(app: AppHandle) -> Result<(), String> {
    stub::start_listener(app)
}

/// 在 refresh_runtime 中于 snippet 快捷键重注册之后调用。
pub fn refresh_trigger(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        Ok(())
    }
    #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
    {
        global_shortcut_listener::refresh_trigger(app)
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        all(unix, not(target_os = "macos"))
    )))]
    {
        let _ = app;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_abbrev_after_correction() {
        assert_eq!(
            parse_abbrev_from_context("prefix :!"),
            Some("!".to_string())
        );
        assert_eq!(
            parse_abbrev_from_context("hello :1234"),
            Some("1234".to_string())
        );
    }

    #[test]
    fn parse_uses_nearest_colon_within_lookback() {
        let text = "ratio:50 and :!";
        assert_eq!(parse_abbrev_from_context(text), Some("!".to_string()));
    }

    #[test]
    fn parse_rejects_empty_or_invalid() {
        assert_eq!(parse_abbrev_from_context(":"), None);
        assert_eq!(parse_abbrev_from_context("no colon here"), None);
        assert_eq!(parse_abbrev_from_context(":bad ab"), None);
    }

    #[test]
    fn parse_rejects_overlong_abbrev() {
        let long = "a".repeat(MAX_ABBREV_LEN + 1);
        assert_eq!(parse_abbrev_from_context(&format!(":{long}")), None);
    }

    #[test]
    fn lookback_truncates_distant_colon() {
        let distant = format!(":old{}", "x".repeat(LOOKBACK_LEN + 5));
        assert_eq!(parse_abbrev_from_context(&distant), None);

        let near = format!("{}:ok", "x".repeat(LOOKBACK_LEN));
        assert_eq!(parse_abbrev_from_context(&near), Some("ok".to_string()));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn expand_trigger_matches_f12() {
        use core_graphics::event::CGEventFlags;

        trigger::apply_trigger_shortcut("F12");
        assert!(trigger::is_expand_trigger_key(0x6f, CGEventFlags::empty()));
        assert!(!trigger::is_expand_trigger_key(0x29, CGEventFlags::empty()));
    }
}
