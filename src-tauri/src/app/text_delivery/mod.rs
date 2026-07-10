//! 跨应用文本投递：模拟键入，必要时经剪贴板粘贴（不拥有剪贴板历史语义）。

use std::thread;
use std::time::Duration;

use enigo::{Direction, Enigo, Key as EnigoKey, Keyboard, Settings};

use crate::app::clipboard;

const INSERT_VIA_CLIPBOARD_LEN: usize = 200;

pub fn insert_at_focus(content: &str) {
    if content.is_empty() {
        return;
    }

    let text = content.to_string();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(30));
        if text.len() > INSERT_VIA_CLIPBOARD_LEN {
            insert_via_clipboard(&text);
        } else if let Err(err) = type_text(&text) {
            eprintln!("[text_delivery] type_text failed: {err}");
            insert_via_clipboard(&text);
        }
    });
}

fn type_text(text: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|err| format!("{err}"))?;
    enigo
        .text(text)
        .map_err(|err| format!("模拟键入失败：{err}"))
}

fn insert_via_clipboard(text: &str) {
    let Ok(original) = clipboard::set_text_transient(text) else {
        let _ = type_text(text);
        return;
    };

    thread::sleep(Duration::from_millis(20));
    if paste_shortcut(&mut Enigo::new(&Settings::default()).ok()).is_err() {
        let _ = type_text(text);
    }

    thread::sleep(Duration::from_millis(50));
    if let Some(original) = original {
        clipboard::restore_text(&original);
    }
}

fn paste_shortcut(enigo: &mut Option<Enigo>) -> Result<(), String> {
    let Some(enigo) = enigo else {
        return Err("Enigo unavailable".to_string());
    };

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

pub fn delete_chars(count: usize) {
    if count == 0 {
        return;
    }

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(35));
        let Ok(mut enigo) = Enigo::new(&Settings::default()) else {
            return;
        };
        for _ in 0..count {
            let _ = enigo.key(EnigoKey::Backspace, Direction::Click);
            thread::sleep(Duration::from_millis(5));
        }
    });
}

pub fn replace_trigger(abbrev_len: usize, content: &str) {
    delete_chars(abbrev_len.saturating_add(2));
    thread::spawn({
        let content = content.to_string();
        move || {
            thread::sleep(Duration::from_millis(60));
            insert_at_focus(&content);
        }
    });
}
