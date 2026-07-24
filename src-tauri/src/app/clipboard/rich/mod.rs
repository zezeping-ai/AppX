//! 跨平台富文本剪贴板（HTML / RTF）读写。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RichFormats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtf: Option<Vec<u8>>,
}

impl RichFormats {
    pub fn has_content(&self) -> bool {
        self.html.as_ref().is_some_and(|s| !s.trim().is_empty())
            || self.rtf.as_ref().is_some_and(|b| !b.is_empty())
    }

    pub fn is_empty(&self) -> bool {
        !self.has_content()
    }
}

/// 读取系统剪贴板富文本（已加 pasteboard 锁）。
#[allow(dead_code)]
pub fn read_system() -> RichFormats {
    crate::app::clipboard::with_pasteboard_lock(read_system_unlocked)
}

/// 读取富文本；调用方必须已持有 pasteboard 锁（例如 `read_once` 内部）。
pub fn read_system_unlocked() -> RichFormats {
    #[cfg(target_os = "macos")]
    {
        macos::read_unlocked()
    }
    #[cfg(target_os = "windows")]
    {
        windows::read_unlocked()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        linux::read_unlocked()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    {
        RichFormats::default()
    }
}

/// 写入系统剪贴板富文本（锁 + 抑制入库）。
pub fn write_system(formats: &RichFormats, plain_text: Option<&str>) -> Result<(), String> {
    if formats.is_empty() {
        return Err("富文本内容缺失".to_string());
    }
    crate::app::clipboard::with_pasteboard_lock(|| {
        crate::app::clipboard::with_record_suppressed(|| {
            write_system_unlocked(formats, plain_text)
        })
    })
}

/// 已持锁写入；不抑制、不打 Transient（由调用方 / transient 负责）。
pub(crate) fn write_system_unlocked(
    formats: &RichFormats,
    plain_text: Option<&str>,
) -> Result<(), String> {
    if formats.is_empty() {
        return Err("富文本内容缺失".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        macos::write_unlocked(formats, plain_text)
    }
    #[cfg(target_os = "windows")]
    {
        windows::write_unlocked(formats, plain_text)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        linux::write_unlocked(formats, plain_text)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    {
        let _ = plain_text;
        Err("当前平台不支持富文本剪贴板".to_string())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use objc2_app_kit::{
        NSPasteboard, NSPasteboardTypeHTML, NSPasteboardTypeRTF, NSPasteboardTypeRTFD,
        NSPasteboardTypeString,
    };
    use objc2_foundation::{NSData, NSString};

    use super::RichFormats;

    fn read_type(pasteboard: &NSPasteboard, ty: &objc2_app_kit::NSPasteboardType) -> Option<Vec<u8>> {
        let data = pasteboard.dataForType(ty)?;
        // SAFETY: pasteboard data is immutable for the duration of this call.
        unsafe { Some(data.as_bytes_unchecked().to_vec()) }
    }

    pub fn read_unlocked() -> RichFormats {
        let pasteboard = NSPasteboard::generalPasteboard();
        let html = unsafe { read_type(&pasteboard, NSPasteboardTypeHTML) }
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .filter(|s| !s.trim().is_empty());
        let rtf = unsafe { read_type(&pasteboard, NSPasteboardTypeRTF) }
            .or_else(|| unsafe { read_type(&pasteboard, NSPasteboardTypeRTFD) })
            .filter(|b| !b.is_empty());
        RichFormats { html, rtf }
    }

    pub fn write_unlocked(formats: &RichFormats, plain_text: Option<&str>) -> Result<(), String> {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();

        if let Some(html) = formats.html.as_ref().filter(|s| !s.trim().is_empty()) {
            let data = NSData::with_bytes(html.as_bytes());
            if !unsafe { pasteboard.setData_forType(Some(&data), NSPasteboardTypeHTML) } {
                return Err("写入 HTML 剪贴板失败".to_string());
            }
        }

        if let Some(rtf) = formats.rtf.as_ref().filter(|b| !b.is_empty()) {
            let data = NSData::with_bytes(rtf);
            if !unsafe { pasteboard.setData_forType(Some(&data), NSPasteboardTypeRTF) } {
                return Err("写入 RTF 剪贴板失败".to_string());
            }
        }

        if let Some(text) = plain_text.filter(|s| !s.is_empty()) {
            let ns = NSString::from_str(text);
            if !unsafe { pasteboard.setString_forType(&ns, NSPasteboardTypeString) } {
                return Err("写入纯文本剪贴板失败".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::ffi::c_void;
    use std::ptr;

    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, RegisterClipboardFormatW,
        SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
    };

    use super::RichFormats;

    fn register_format(name: &str) -> u32 {
        let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
        unsafe { RegisterClipboardFormatW(PCWSTR(wide.as_ptr())) }
    }

    fn read_format(format: u32) -> Option<Vec<u8>> {
        unsafe {
            OpenClipboard(None).ok()?;
            let handle = GetClipboardData(format).ok()?;
            let ptr = GlobalLock(HGLOBAL(handle.0 as *mut c_void));
            if ptr.is_null() {
                let _ = CloseClipboard();
                return None;
            }
            let size = GlobalSize(HGLOBAL(handle.0 as *mut c_void));
            let mut out = vec![0u8; size];
            ptr::copy_nonoverlapping(ptr as *const u8, out.as_mut_ptr(), size);
            let _ = GlobalUnlock(HGLOBAL(handle.0 as *mut c_void));
            let _ = CloseClipboard();
            Some(out)
        }
    }

    fn write_format(format: u32, bytes: &[u8]) -> Result<(), String> {
        unsafe {
            let handle = GlobalAlloc(GMEM_MOVEABLE, bytes.len())
                .map_err(|e| format!("分配剪贴板内存失败：{e}"))?;
            let locked = GlobalLock(handle);
            if locked.is_null() {
                let _ = GlobalFree(handle);
                return Err("锁定剪贴板内存失败".to_string());
            }
            ptr::copy_nonoverlapping(bytes.as_ptr(), locked as *mut u8, bytes.len());
            let _ = GlobalUnlock(handle);
            if SetClipboardData(format, HANDLE(handle.0)).is_err() {
                let _ = GlobalFree(handle);
                return Err("写入剪贴板失败".to_string());
            }
            Ok(())
        }
    }

    pub fn read_unlocked() -> RichFormats {
        let html_format = register_format("HTML Format");
        let rtf_format = register_format("Rich Text Format");
        let html = read_format(html_format)
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .filter(|s| !s.trim().is_empty());
        let rtf = read_format(rtf_format).filter(|b| !b.is_empty());
        RichFormats { html, rtf }
    }

    pub fn write_unlocked(formats: &RichFormats, plain_text: Option<&str>) -> Result<(), String> {
        unsafe {
            OpenClipboard(None).map_err(|e| format!("打开剪贴板失败：{e}"))?;
            EmptyClipboard()
                .map_err(|e| format!("清空剪贴板失败：{e}"))?;

            let html_format = register_format("HTML Format");
            let rtf_format = register_format("Rich Text Format");
            let unicode_format = 13u32; // CF_UNICODETEXT

            if let Some(html) = formats.html.as_ref().filter(|s| !s.trim().is_empty()) {
                write_format(html_format, html.as_bytes())?;
            }
            if let Some(rtf) = formats.rtf.as_ref().filter(|b| !b.is_empty()) {
                write_format(rtf_format, rtf)?;
            }
            if let Some(text) = plain_text.filter(|s| !s.is_empty()) {
                let wide: Vec<u16> = text.encode_utf16().chain([0]).collect();
                let mut bytes = Vec::with_capacity(wide.len() * 2);
                for unit in &wide {
                    bytes.extend_from_slice(&unit.to_le_bytes());
                }
                write_format(unicode_format, &bytes)?;
            }

            CloseClipboard()
                .map_err(|e| format!("关闭剪贴板失败：{e}"))?;
            Ok(())
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod linux {
    use std::io::Write;
    use std::process::{Command, Stdio};

    use arboard::Clipboard;

    use super::RichFormats;

    fn read_mime(mime: &str) -> Option<Vec<u8>> {
        for (bin, args) in [
            ("wl-paste", vec!["-t", mime, "-n"]),
            ("xclip", vec!["-selection", "clipboard", "-t", mime]),
        ] {
            let output = Command::new(bin).args(&args).output().ok()?;
            if output.status.success() && !output.stdout.is_empty() {
                return Some(output.stdout);
            }
        }
        None
    }

    fn write_mime(mime: &str, bytes: &[u8]) -> Result<(), String> {
        if let Ok(mut child) = Command::new("wl-copy")
            .args(["-t", mime])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin
                    .write_all(bytes)
                    .map_err(|e| format!("写入 wl-copy 失败：{e}"))?;
            }
            if child
                .wait()
                .map_err(|e| format!("等待 wl-copy 失败：{e}"))?
                .success()
            {
                return Ok(());
            }
        }

        if Command::new("xclip")
            .args(["-selection", "clipboard", "-t", mime])
            .stdin(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(bytes)?;
                }
                child.wait()?;
                Ok(())
            })
            .is_ok()
        {
            return Ok(());
        }

        Err(format!("写入 {mime} 剪贴板失败（需要 wl-copy 或 xclip）"))
    }

    pub fn read_unlocked() -> RichFormats {
        let html = read_mime("text/html")
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .filter(|s| !s.trim().is_empty());
        let rtf = read_mime("text/rtf").filter(|b| !b.is_empty());
        RichFormats { html, rtf }
    }

    pub fn write_unlocked(formats: &RichFormats, plain_text: Option<&str>) -> Result<(), String> {
        let wrote_html = formats
            .html
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        if let Some(html) = formats.html.as_ref().filter(|s| !s.trim().is_empty()) {
            write_mime("text/html", html.as_bytes())?;
        }
        if let Some(rtf) = formats.rtf.as_ref().filter(|b| !b.is_empty()) {
            write_mime("text/rtf", rtf)?;
        }
        if let Some(text) = plain_text.filter(|s| !s.is_empty()) {
            if !wrote_html {
                Clipboard::new()
                    .map_err(|err| format!("无法访问剪贴板：{err}"))?
                    .set_text(text.to_string())
                    .map_err(|err| format!("写入剪贴板失败：{err}"))?;
            }
        }
        Ok(())
    }
}
