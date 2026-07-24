use std::thread;
use std::time::Duration;

use tauri::AppHandle;

use crate::app::clipboard;
use crate::app::clipboard::rich;
use crate::app::focus_target;
use crate::app::text_delivery;

use super::super::history::{load_payload_for_apply, touch_item};
use super::super::model::{ApplyAction, ApplyFormat, PayloadKind};
use super::super::palette::hide_palette_window;
use super::super::state::ClipboardAssistantState;

pub fn apply_item(
    app: &AppHandle,
    state: &ClipboardAssistantState,
    id: i64,
    action: ApplyAction,
    format: ApplyFormat,
) -> Result<(), String> {
    let (kind, text, paths, blob, rich_formats) = load_payload_for_apply(state, id)?;
    let settings = state.settings()?;
    let plain_text = text.clone().or_else(|| {
        blob.as_ref()
            .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
    });

    if action == ApplyAction::Copy {
        write_to_clipboard(kind, format, plain_text.as_deref(), paths.as_deref(), blob.as_deref(), rich_formats.as_ref())?;
        touch_item(state, id, None)?;
        super::super::sounds::play(app, super::super::sounds::SoundKind::Copy, None, false);
        if settings.auto_hide_on_paste {
            hide_palette_window(app)?;
        }
        return Ok(());
    }

    hide_palette_window(app)?;
    focus_target::restore();
    super::super::sounds::play(app, super::super::sounds::SoundKind::Paste, None, false);

    if format == ApplyFormat::Rich
        && rich_formats.as_ref().is_some_and(|formats| formats.has_content())
    {
        rich::write_system(rich_formats.as_ref().unwrap(), plain_text.as_deref())?;
        text_delivery::trigger_paste();
        touch_item(state, id, None)?;
        return Ok(());
    }

    match kind {
        PayloadKind::Inline => {
            let Some(text) = text else {
                return Err("文本内容缺失".to_string());
            };
            deliver_text(text);
        }
        PayloadKind::Blob => {
            if let Some(ref bytes) = blob {
                if let Ok(text) = std::str::from_utf8(bytes) {
                    deliver_text(text.to_string());
                } else {
                    clipboard::image::write_image_bytes(bytes)?;
                    text_delivery::trigger_paste();
                }
            }
        }
        PayloadKind::FileRef => {
            clipboard::files::write_file_paths(
                paths.as_deref().ok_or_else(|| "文件路径缺失".to_string())?,
            )?;
            text_delivery::trigger_paste();
        }
    }

    touch_item(state, id, None)?;
    Ok(())
}

fn write_to_clipboard(
    kind: PayloadKind,
    format: ApplyFormat,
    text: Option<&str>,
    paths: Option<&[String]>,
    blob: Option<&[u8]>,
    rich_formats: Option<&crate::app::clipboard::rich::RichFormats>,
) -> Result<(), String> {
    let use_rich = format != ApplyFormat::Plain
        && rich_formats.is_some_and(|formats| formats.has_content());
    if use_rich {
        return rich::write_system(rich_formats.unwrap(), text);
    }

    match kind {
        PayloadKind::Inline => {
            let text = text.ok_or_else(|| "文本内容缺失".to_string())?;
            clipboard::set_text_persist(text)
        }
        PayloadKind::Blob => {
            let bytes = blob.ok_or_else(|| "二进制内容缺失".to_string())?;
            if let Ok(text) = std::str::from_utf8(bytes) {
                return clipboard::set_text_persist(text);
            }
            clipboard::image::write_image_bytes(bytes)
        }
        PayloadKind::FileRef => {
            let paths = paths.ok_or_else(|| "文件路径缺失".to_string())?;
            clipboard::files::write_file_paths(paths)
        }
    }
}

fn deliver_text(text: String) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        text_delivery::insert_at_focus(&text);
    });
}
