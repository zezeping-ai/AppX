use std::thread;
use std::time::Duration;

use arboard::{Clipboard, ImageData};
use tauri::AppHandle;

use crate::app::clipboard;
use crate::app::focus_target;
use crate::app::text_delivery;

use super::super::history::{load_payload_for_apply, touch_item};
use super::super::model::{ApplyAction, PayloadKind};
use super::super::palette::hide_palette_window;
use super::super::state::ClipboardAssistantState;

pub fn apply_item(
    app: &AppHandle,
    state: &ClipboardAssistantState,
    id: i64,
    action: ApplyAction,
    _plain_text: bool,
) -> Result<(), String> {
    let (kind, text, paths, blob) = load_payload_for_apply(state, id)?;
    let settings = state.settings()?;

    if action == ApplyAction::Copy {
        write_clipboard(kind, text.as_deref(), paths.as_deref(), blob.as_deref())?;
        touch_item(state, id)?;
        if settings.auto_hide_on_paste {
            hide_palette_window(app)?;
        }
        return Ok(());
    }

    // 粘贴：文本直接模拟键入，不先写入系统剪贴板，避免监控产生重复条目。
    hide_palette_window(app)?;
    focus_target::restore();

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
                    write_clipboard(kind, None, None, Some(bytes))?;
                    text_delivery::trigger_paste();
                }
            }
        }
        PayloadKind::FileRef => {
            write_clipboard(kind, None, paths.as_deref(), None)?;
            text_delivery::trigger_paste();
        }
    }

    touch_item(state, id)?;
    Ok(())
}

fn deliver_text(text: String) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        text_delivery::insert_at_focus(&text);
    });
}

fn write_clipboard(
    kind: PayloadKind,
    text: Option<&str>,
    paths: Option<&[String]>,
    blob: Option<&[u8]>,
) -> Result<(), String> {
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
            set_image_bytes(bytes)
        }
        PayloadKind::FileRef => {
            let paths = paths.ok_or_else(|| "文件路径缺失".to_string())?;
            set_file_paths(paths)
        }
    }
}

fn set_image_bytes(bytes: &[u8]) -> Result<(), String> {
    clipboard::with_pasteboard_lock(|| {
        clipboard::with_record_suppressed(|| {
            let img = image::load_from_memory(bytes).map_err(|e| format!("解码图片失败：{e}"))?;
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            Clipboard::new()
                .map_err(|e| format!("无法访问剪贴板：{e}"))?
                .set_image(ImageData {
                    width: w as usize,
                    height: h as usize,
                    bytes: rgba.into_raw().into(),
                })
                .map_err(|e| format!("写入图片剪贴板失败：{e}"))
        })
    })
}

fn set_file_paths(paths: &[String]) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        set_file_paths_macos(paths)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let joined = paths.join("\n");
        clipboard::set_text_persist(&joined)
    }
}

#[cfg(target_os = "macos")]
fn set_file_paths_macos(paths: &[String]) -> Result<(), String> {
    use std::process::Command;

    clipboard::with_pasteboard_lock(|| {
        clipboard::with_record_suppressed(|| {
            let quoted: Vec<String> = paths
                .iter()
                .map(|p| format!("POSIX file {:?}", p))
                .collect();
            let script = if quoted.len() == 1 {
                format!("set the clipboard to {}", quoted[0])
            } else {
                format!("set the clipboard to {{{}}}", quoted.join(", "))
            };
            let output = Command::new("osascript")
                .args(["-e", &script])
                .output()
                .map_err(|e| format!("调用 osascript 失败：{e}"))?;
            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("写入文件剪贴板失败：{err}"));
            }
            Ok(())
        })
    })
}
