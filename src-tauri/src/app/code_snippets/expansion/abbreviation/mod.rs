//! `:缩写;` 全局缩写展开。macOS 使用 CGEventTap（仅读 keycode，不调用 TSM）。

#[cfg(target_os = "macos")]
mod foreground;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod stub;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};

use crate::app::text_delivery::replace_trigger;
use crate::app::code_snippets::registry::SnippetRegistry;
use crate::app::code_snippets::settings::is_inline_expansion_enabled;

const MAX_ABBREV_LEN: usize = 32;
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(5);

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

/// macOS 虚拟键码（HIToolbox/Events.h）
pub(crate) mod keycode {
    pub const ANSI_SEMICOLON: i64 = 0x29; // `;` / Shift → `:`
    pub const DELETE: i64 = 0x33;
    pub const ESCAPE: i64 = 0x35;
}

#[derive(Debug, Clone)]
enum CaptureState {
    Idle,
    Capturing {
        buffer: String,
        started: Instant,
    },
}

pub(crate) struct ListenerState {
    capture: CaptureState,
}

impl Default for ListenerState {
    fn default() -> Self {
        Self {
            capture: CaptureState::Idle,
        }
    }
}

pub(crate) enum KeyAction {
    Pass,
}

pub(crate) fn handle_keydown(
    app: &AppHandle,
    state: &Mutex<ListenerState>,
    shift_held: bool,
    keycode: i64,
) -> KeyAction {
    if keycode == keycode::DELETE {
        return with_capture_state(state, |guard| handle_backspace(guard));
    }

    if keycode == keycode::ESCAPE {
        return with_capture_state(state, |guard| {
            guard.capture = CaptureState::Idle;
            KeyAction::Pass
        });
    }

    if is_trigger_suffix(keycode, shift_held) {
        return with_capture_state(state, |guard| handle_expand_trigger(app, guard));
    }

    if is_trigger_prefix(keycode, shift_held) {
        return with_capture_state(state, |guard| {
            start_capture(guard);
            KeyAction::Pass
        });
    }

    with_capture_state(state, |guard| match &mut guard.capture {
        CaptureState::Idle => KeyAction::Pass,
        CaptureState::Capturing { .. } => {
            if let Some(ch) = keycode_to_abbrev_char(keycode, shift_held) {
                push_abbrev_char(guard, ch)
            } else {
                guard.capture = CaptureState::Idle;
                KeyAction::Pass
            }
        }
    })
}

pub(crate) fn handle_text_char(
    app: &AppHandle,
    state: &Mutex<ListenerState>,
    ch: char,
) -> KeyAction {
    if ch == ';' {
        return with_capture_state(state, |guard| handle_expand_trigger(app, guard));
    }

    if ch == ':' {
        return with_capture_state(state, |guard| {
            start_capture(guard);
            KeyAction::Pass
        });
    }

    with_capture_state(state, |guard| match &guard.capture {
        CaptureState::Idle => KeyAction::Pass,
        CaptureState::Capturing { .. } => push_abbrev_char(guard, ch),
    })
}

fn with_capture_state<F>(state: &Mutex<ListenerState>, f: F) -> KeyAction
where
    F: FnOnce(&mut ListenerState) -> KeyAction,
{
    if is_expansion_paused() || !is_inline_expansion_enabled() {
        return KeyAction::Pass;
    }

    let mut guard = match state.lock() {
        Ok(guard) => guard,
        Err(_) => return KeyAction::Pass,
    };

    expire_capture_if_needed(&mut guard);
    f(&mut guard)
}

fn expire_capture_if_needed(guard: &mut ListenerState) {
    if let CaptureState::Capturing { started, .. } = &guard.capture {
        if started.elapsed() > CAPTURE_TIMEOUT {
            guard.capture = CaptureState::Idle;
        }
    }
}

fn start_capture(guard: &mut ListenerState) {
    guard.capture = CaptureState::Capturing {
        buffer: String::new(),
        started: Instant::now(),
    };
}

fn push_abbrev_char(guard: &mut ListenerState, ch: char) -> KeyAction {
    let CaptureState::Capturing { buffer, .. } = &mut guard.capture else {
        return KeyAction::Pass;
    };
    if !is_allowed_abbrev_char(ch) {
        guard.capture = CaptureState::Idle;
        return KeyAction::Pass;
    }
    if buffer.len() >= MAX_ABBREV_LEN {
        guard.capture = CaptureState::Idle;
        return KeyAction::Pass;
    }
    buffer.push(ch);
    KeyAction::Pass
}

fn handle_backspace(guard: &mut ListenerState) -> KeyAction {
    let CaptureState::Capturing { buffer, .. } = &mut guard.capture else {
        return KeyAction::Pass;
    };

    if buffer.pop().is_none() {
        guard.capture = CaptureState::Idle;
    }
    KeyAction::Pass
}

fn handle_expand_trigger(app: &AppHandle, guard: &mut ListenerState) -> KeyAction {
    let CaptureState::Capturing { buffer, .. } = guard.capture.clone() else {
        return KeyAction::Pass;
    };

    guard.capture = CaptureState::Idle;

    if buffer.is_empty() {
        return KeyAction::Pass;
    }

    let Some(registry) = app.try_state::<SnippetRegistry>() else {
        log::warn!("[code_snippets] registry unavailable for `{buffer}`");
        return KeyAction::Pass;
    };

    let snapshot = registry.snapshot();
    let Some(entry) = snapshot.by_abbreviation.get(&buffer) else {
        log::warn!(
            "[code_snippets] no snippet for `{buffer}` (registered: {:?})",
            snapshot.by_abbreviation.keys().collect::<Vec<_>>()
        );
        return KeyAction::Pass;
    };

    log::info!("[code_snippets] expanding `{buffer}`");
    replace_trigger(buffer.len(), &entry.content);
    KeyAction::Pass
}

fn is_trigger_suffix(keycode: i64, shift_held: bool) -> bool {
    !shift_held && keycode == keycode::ANSI_SEMICOLON
}

fn is_trigger_prefix(keycode: i64, shift_held: bool) -> bool {
    shift_held && keycode == keycode::ANSI_SEMICOLON
}

fn keycode_to_abbrev_char(keycode: i64, shift_held: bool) -> Option<char> {
    if !shift_held {
        if let Some(ch) = keycode_to_letter(keycode) {
            return Some(ch);
        }
    }

    match (keycode, shift_held) {
        (0x1B, false) => Some('-'),
        (0x1B, true) => Some('_'),
        (0x12, false) => Some('1'),
        (0x12, true) => Some('!'),
        (0x13, false) => Some('2'),
        (0x14, false) => Some('3'),
        (0x14, true) => Some('#'),
        (0x15, false) => Some('4'),
        (0x17, false) => Some('5'),
        (0x17, true) => Some('%'),
        (0x16, false) => Some('6'),
        (0x16, true) => Some('^'),
        (0x1A, false) => Some('7'),
        (0x1A, true) => Some('&'),
        (0x1C, false) => Some('8'),
        (0x1C, true) => Some('*'),
        (0x19, false) => Some('9'),
        (0x19, true) => Some('('),
        (0x1D, false) => Some('0'),
        (0x1D, true) => Some(')'),
        (0x18, false) => Some('='),
        (0x18, true) => Some('+'),
        (0x2B, false) => Some(','),
        (0x2B, true) => Some('<'),
        (0x2F, false) => Some('.'),
        (0x2F, true) => Some('>'),
        (0x2C, false) => Some('/'),
        (0x2C, true) => Some('?'),
        _ => None,
    }
}

fn keycode_to_letter(keycode: i64) -> Option<char> {
    match keycode {
        0x00 => Some('a'),
        0x01 => Some('s'),
        0x02 => Some('d'),
        0x03 => Some('f'),
        0x04 => Some('h'),
        0x05 => Some('g'),
        0x06 => Some('z'),
        0x07 => Some('x'),
        0x08 => Some('c'),
        0x09 => Some('v'),
        0x0B => Some('b'),
        0x0C => Some('q'),
        0x0D => Some('w'),
        0x0E => Some('e'),
        0x0F => Some('r'),
        0x10 => Some('y'),
        0x11 => Some('t'),
        0x2D => Some('n'),
        0x2E => Some('m'),
        _ => None,
    }
}

fn is_allowed_abbrev_char(ch: char) -> bool {
    if ch == ':' || ch == ';' {
        return false;
    }
    ch.is_ascii_lowercase()
        || ch.is_ascii_digit()
        || matches!(
            ch,
            '-' | '_' | '!' | '#' | '%' | '^' | '&' | '*' | '(' | ')' | '+' | '=' | '.'
                | ',' | '?' | '/' | '<' | '>'
        )
}

#[cfg(target_os = "macos")]
pub fn start_listener(app: AppHandle) -> Result<(), String> {
    macos::start_listener(app)
}

#[cfg(not(target_os = "macos"))]
pub fn start_listener(app: AppHandle) -> Result<(), String> {
    stub::start_listener(app)
}
