//! 将 Tauri global-shortcut 字符串解析为可匹配的触发键配置。

#[cfg(target_os = "macos")]
use once_cell::sync::Lazy;
use once_cell::sync::Lazy as TriggerLazy;
use std::sync::RwLock;

#[cfg(target_os = "macos")]
use keyboard_types::{Code, Modifiers};
#[cfg(target_os = "macos")]
use tauri_plugin_global_shortcut::Shortcut;

#[cfg(target_os = "macos")]
use core_graphics::event::CGEventFlags;

const DEFAULT_TRIGGER: &str = "F12";

static TRIGGER_SHORTCUT: TriggerLazy<RwLock<String>> =
    TriggerLazy::new(|| RwLock::new(DEFAULT_TRIGGER.to_string()));

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, Default)]
struct TriggerConfig {
    keycode: Option<i64>,
    shift: bool,
    control: bool,
    alt: bool,
    command: bool,
}

#[cfg(target_os = "macos")]
static TRIGGER: Lazy<RwLock<TriggerConfig>> = Lazy::new(|| {
    RwLock::new(parse_trigger(DEFAULT_TRIGGER).unwrap_or_default())
});

pub fn apply_trigger_shortcut(raw: &str) {
    let shortcut = raw.trim();
    let stored = if shortcut.is_empty() {
        DEFAULT_TRIGGER.to_string()
    } else {
        shortcut.to_string()
    };
    if let Ok(mut guard) = TRIGGER_SHORTCUT.write() {
        *guard = stored;
    }

    #[cfg(target_os = "macos")]
    {
        let parsed = parse_trigger(raw).unwrap_or_else(|_| parse_trigger(DEFAULT_TRIGGER).unwrap());
        if let Ok(mut guard) = TRIGGER.write() {
            *guard = parsed;
        }
    }
}

#[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
pub fn current_trigger_shortcut() -> String {
    TRIGGER_SHORTCUT
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| DEFAULT_TRIGGER.to_string())
}

#[cfg(target_os = "macos")]
pub fn is_expand_trigger_key(keycode: i64, flags: CGEventFlags) -> bool {
    let config = TRIGGER.read().map(|c| *c).unwrap_or_default();
    let Some(expected) = config.keycode else {
        return false;
    };
    if keycode != expected {
        return false;
    }

    let base = CGEventFlags::CGEventFlagShift
        | CGEventFlags::CGEventFlagControl
        | CGEventFlags::CGEventFlagAlternate
        | CGEventFlags::CGEventFlagCommand;

    let active = flags & base;
    let required = trigger_flags(&config);
    active == required
}

#[cfg(target_os = "macos")]
fn trigger_flags(config: &TriggerConfig) -> CGEventFlags {
    let mut flags = CGEventFlags::empty();
    if config.shift {
        flags |= CGEventFlags::CGEventFlagShift;
    }
    if config.control {
        flags |= CGEventFlags::CGEventFlagControl;
    }
    if config.alt {
        flags |= CGEventFlags::CGEventFlagAlternate;
    }
    if config.command {
        flags |= CGEventFlags::CGEventFlagCommand;
    }
    flags
}

#[cfg(target_os = "macos")]
fn parse_trigger(raw: &str) -> Result<TriggerConfig, String> {
    let shortcut = raw.trim();
    if shortcut.is_empty() {
        return Err("empty trigger shortcut".into());
    }

    let hotkey = Shortcut::try_from(shortcut)
        .map_err(|err| format!("快捷键 `{shortcut}` 无效：{err}"))?;

    Ok(TriggerConfig {
        keycode: code_to_keycode(hotkey.key).map(|code| code as i64),
        shift: hotkey.mods.contains(Modifiers::SHIFT),
        control: hotkey.mods.contains(Modifiers::CONTROL),
        alt: hotkey.mods.contains(Modifiers::ALT),
        command: hotkey.mods.intersects(Modifiers::SUPER | Modifiers::META),
    })
}

/// macOS 虚拟键码（HIToolbox/Events.h），与 global-hotkey 一致。
#[cfg(target_os = "macos")]
fn code_to_keycode(code: Code) -> Option<u32> {
    Some(match code {
        Code::KeyA => 0x00,
        Code::KeyS => 0x01,
        Code::KeyD => 0x02,
        Code::KeyF => 0x03,
        Code::KeyH => 0x04,
        Code::KeyG => 0x05,
        Code::KeyZ => 0x06,
        Code::KeyX => 0x07,
        Code::KeyC => 0x08,
        Code::KeyV => 0x09,
        Code::KeyB => 0x0b,
        Code::KeyQ => 0x0c,
        Code::KeyW => 0x0d,
        Code::KeyE => 0x0e,
        Code::KeyR => 0x0f,
        Code::KeyY => 0x10,
        Code::KeyT => 0x11,
        Code::Digit1 => 0x12,
        Code::Digit2 => 0x13,
        Code::Digit3 => 0x14,
        Code::Digit4 => 0x15,
        Code::Digit6 => 0x16,
        Code::Digit5 => 0x17,
        Code::Equal => 0x18,
        Code::Digit9 => 0x19,
        Code::Digit7 => 0x1a,
        Code::Minus => 0x1b,
        Code::Digit8 => 0x1c,
        Code::Digit0 => 0x1d,
        Code::BracketRight => 0x1e,
        Code::KeyO => 0x1f,
        Code::KeyU => 0x20,
        Code::BracketLeft => 0x21,
        Code::KeyI => 0x22,
        Code::KeyP => 0x23,
        Code::Enter => 0x24,
        Code::KeyL => 0x25,
        Code::KeyJ => 0x26,
        Code::Quote => 0x27,
        Code::KeyK => 0x28,
        Code::Semicolon => 0x29,
        Code::Backslash => 0x2a,
        Code::Comma => 0x2b,
        Code::Slash => 0x2c,
        Code::KeyN => 0x2d,
        Code::KeyM => 0x2e,
        Code::Period => 0x2f,
        Code::Tab => 0x30,
        Code::Space => 0x31,
        Code::Backquote => 0x32,
        Code::Backspace => 0x33,
        Code::Escape => 0x35,
        Code::F17 => 0x40,
        Code::F18 => 0x4f,
        Code::F19 => 0x50,
        Code::F20 => 0x5a,
        Code::F5 => 0x60,
        Code::F6 => 0x61,
        Code::F7 => 0x62,
        Code::F3 => 0x63,
        Code::F8 => 0x64,
        Code::F9 => 0x65,
        Code::F11 => 0x67,
        Code::F13 => 0x69,
        Code::F16 => 0x6a,
        Code::F14 => 0x6b,
        Code::F10 => 0x6d,
        Code::F12 => 0x6f,
        Code::F15 => 0x71,
        Code::Insert => 0x72,
        Code::Home => 0x73,
        Code::PageUp => 0x74,
        Code::Delete => 0x75,
        Code::F4 => 0x76,
        Code::End => 0x77,
        Code::F2 => 0x78,
        Code::PageDown => 0x79,
        Code::F1 => 0x7a,
        Code::ArrowLeft => 0x7b,
        Code::ArrowRight => 0x7c,
        Code::ArrowDown => 0x7d,
        Code::ArrowUp => 0x7e,
        Code::CapsLock => 0x39,
        _ => return None,
    })
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::*;

    #[test]
    fn parse_f12_default() {
        let config = parse_trigger("F12").unwrap();
        assert_eq!(config.keycode, Some(0x6f));
        assert!(!config.shift);
        assert!(!config.command);
    }

    #[test]
    fn parse_shortcut_with_modifiers() {
        let config = parse_trigger("CommandOrControl+Shift+KeyH").unwrap();
        assert_eq!(config.keycode, Some(0x04));
        assert!(config.shift);
        assert!(config.command);
    }
}
