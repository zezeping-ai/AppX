//! 焦点投递：编排「临时写板 → 粘贴快捷键 → 恢复」。
//! 正文一律走剪贴板粘贴；enigo 仅用于 Cmd/Ctrl+V 与退格，不再模拟键入正文。

use std::thread;
use std::time::Duration;

use enigo::{Direction, Enigo, Key as EnigoKey, Keyboard, Settings};

use crate::app::clipboard;
use crate::app::clipboard::rich::RichFormats;
use crate::app::clipboard::transient::PasteboardBackup;

#[cfg(target_os = "macos")]
mod macos;

/// 焦点切回后稍候再写板（hide palette / activate 需要时间）。
const FOCUS_SETTLE_MS: u64 = 120;
/// Cmd/Ctrl+V 前留给系统同步剪贴板。
const PRE_PASTE_MS: u64 = 20;
/// 粘贴后多久恢复原板（WebView/TipTap 读板偏慢）。
const RESTORE_DELAY_MS: u64 = 150;

/// 剪贴板已写好（图片/文件）：只触发粘贴，不恢复原板。
pub fn trigger_paste() {
    thread::spawn(|| {
        let _suppress = clipboard::suppress_recording();
        thread::sleep(Duration::from_millis(FOCUS_SETTLE_MS));
        let _ = paste_shortcut();
        thread::sleep(Duration::from_millis(RESTORE_DELAY_MS));
    });
}

/// 纯文本：Transient 写板 → 粘贴 → 恢复。
pub fn insert_at_focus(content: &str) {
    if content.is_empty() {
        return;
    }
    let text = content.to_string();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(FOCUS_SETTLE_MS));
        run_transient_paste(|| clipboard::transient::write_text_for_paste(&text));
    });
}

/// 富文本：Transient 写板 → 粘贴 → 恢复；无富文本时退回纯文本通路。
pub fn insert_rich_at_focus(formats: RichFormats, plain_text: Option<String>) {
    if formats.is_empty() {
        if let Some(text) = plain_text {
            insert_at_focus(&text);
        }
        return;
    }
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(FOCUS_SETTLE_MS));
        run_transient_paste(|| {
            clipboard::transient::write_rich_for_paste(&formats, plain_text.as_deref())
        });
    });
}

/// 粘贴会话：一层 suppress 覆盖写板 → 粘贴 → 恢复。
fn run_transient_paste(write: impl FnOnce() -> Result<PasteboardBackup, String>) {
    let _suppress = clipboard::suppress_recording();
    let Ok(backup) = write() else {
        return;
    };

    thread::sleep(Duration::from_millis(PRE_PASTE_MS));
    if paste_shortcut().is_err() {
        clipboard::transient::restore_backup(&backup);
        return;
    }
    thread::sleep(Duration::from_millis(RESTORE_DELAY_MS));
    clipboard::transient::restore_backup(&backup);
}

fn paste_shortcut() -> Result<(), String> {
    run_keyboard(|| {
        let mut enigo = Enigo::new(&Settings::default()).map_err(|err| format!("{err}"))?;
        paste_shortcut_with(&mut enigo)
    })
}

fn paste_shortcut_with(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        enigo
            .key(EnigoKey::Meta, Direction::Press)
            .map_err(|err| format!("{err}"))?;
        enigo
            .key(EnigoKey::Unicode('v'), Direction::Click)
            .map_err(|err| format!("{err}"))?;
        enigo
            .key(EnigoKey::Meta, Direction::Release)
            .map_err(|err| format!("{err}"))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(EnigoKey::Control, Direction::Press)
            .map_err(|err| format!("{err}"))?;
        enigo
            .key(EnigoKey::Unicode('v'), Direction::Click)
            .map_err(|err| format!("{err}"))?;
        enigo
            .key(EnigoKey::Control, Direction::Release)
            .map_err(|err| format!("{err}"))?;
    }

    Ok(())
}

/// 缩写展开：退格删除 `:缩写`，再 Transient 粘贴真实内容（与剪切板投递同一套逻辑）。
/// 目标应用已聚焦，无需 focus settle。
pub fn replace_trigger(abbrev_len: usize, content: &str) {
    let delete_count = abbrev_len.saturating_add(1); // `:` + 缩写
    let content = content.to_string();
    thread::spawn(move || {
        delete_chars_sync(delete_count);
        // 等编辑器消化退格后再写板粘贴
        thread::sleep(Duration::from_millis(PRE_PASTE_MS));
        if content.is_empty() {
            return;
        }
        run_transient_paste(|| clipboard::transient::write_text_for_paste(&content));
    });
}

fn delete_chars_sync(count: usize) {
    if count == 0 {
        return;
    }
    let _ = run_keyboard(move || {
        let mut enigo = Enigo::new(&Settings::default()).map_err(|err| format!("{err}"))?;
        for _ in 0..count {
            enigo
                .key(EnigoKey::Backspace, Direction::Click)
                .map_err(|err| format!("{err}"))?;
        }
        Ok::<(), String>(())
    });
}

#[cfg(target_os = "macos")]
fn run_keyboard<R, F>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    macos::run_on_main_thread(f)
}

#[cfg(not(target_os = "macos"))]
fn run_keyboard<R, F>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}
