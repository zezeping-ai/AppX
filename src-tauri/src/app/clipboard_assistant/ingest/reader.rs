use arboard::Clipboard;

use crate::app::clipboard;
use crate::app::clipboard::rich::{self, RichFormats};

#[derive(Debug, Clone)]
pub struct ClipboardRead {
    pub text: Option<String>,
    pub image: Option<ImageCapture>,
    pub files: Option<Vec<String>>,
    pub rich_formats: RichFormats,
}

#[derive(Debug, Clone)]
pub struct ImageCapture {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// 单次读取系统剪贴板（Classifier 前唯一读盘点）。
pub fn read_once() -> Result<ClipboardRead, String> {
    clipboard::with_pasteboard_lock(read_once_inner)
}

fn read_once_inner() -> Result<ClipboardRead, String> {
    #[cfg(target_os = "macos")]
    let files = read_macos_files();
    #[cfg(not(target_os = "macos"))]
    let files: Option<Vec<String>> = None;

    let mut clipboard = Clipboard::new().map_err(|e| format!("无法访问剪贴板：{e}"))?;

    let image = if let Ok(image) = clipboard.get_image() {
        Some(ImageCapture {
            bytes: image.bytes.to_vec(),
            width: image.width as u32,
            height: image.height as u32,
        })
    } else {
        None
    };

    let text = clipboard.get_text().ok().filter(|t| !t.is_empty());
    let rich_formats = rich::read_system_unlocked();

    Ok(ClipboardRead {
        text,
        image,
        files,
        rich_formats,
    })
}

pub fn fingerprint(read: &ClipboardRead) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    if let Some(files) = &read.files {
        hasher.update(b"files:");
        hasher.update(files.join("\n").as_bytes());
    }
    if let Some(img) = &read.image {
        hasher.update(b"img:");
        // 尺寸 + 首尾采样，避免每次哈希整图前 4KB
        hasher.update(img.width.to_le_bytes());
        hasher.update(img.height.to_le_bytes());
        hasher.update(img.bytes.len().to_le_bytes());
        let n = img.bytes.len().min(256);
        hasher.update(&img.bytes[..n]);
        if img.bytes.len() > 256 {
            let start = img.bytes.len() - 256;
            hasher.update(&img.bytes[start..]);
        }
    }
    if let Some(text) = &read.text {
        hasher.update(b"text:");
        hasher.update(text.as_bytes());
    }
    if read.rich_formats.has_content() {
        hasher.update(b"rich:");
        if let Some(html) = &read.rich_formats.html {
            hasher.update(html.as_bytes());
        }
        if let Some(rtf) = &read.rich_formats.rtf {
            hasher.update(&rtf[..rtf.len().min(4096)]);
        }
    }
    if read.files.is_none() && read.image.is_none() && read.text.is_none() {
        hasher.update(b"empty");
    }
    let digest = hasher.finalize();
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(target_os = "macos")]
fn pasteboard_may_contain_files() -> bool {
    use objc2_app_kit::NSPasteboard;

    let pasteboard = NSPasteboard::generalPasteboard();
    let Some(types) = pasteboard.types() else {
        return false;
    };
    // 无文件类型时跳过 osascript；常见：public.file-url / NSFilenamesPboardType
    for ty in types.iter() {
        let s = ty.to_string();
        if s == "public.file-url"
            || s == "NSFilenamesPboardType"
            || s == "com.apple.pasteboard.promised-file-url"
            || s.contains("file-url")
            || s.contains("Filenames")
        {
            return true;
        }
    }
    false
}

#[cfg(target_os = "macos")]
fn read_macos_files() -> Option<Vec<String>> {
    use std::process::Command;

    // 无文件类型时跳过昂贵的 osascript
    if !pasteboard_may_contain_files() {
        return None;
    }

    let output = Command::new("osascript")
        .args([
            "-e",
            r#"
        try
          set fileList to {}
          repeat with itemRef in (the clipboard as list)
            try
              set end of fileList to POSIX path of itemRef
            end try
          end repeat
          if (count of fileList) > 0 then
            return fileList as text
          end if
        end try
        try
          return POSIX path of (the clipboard as alias)
        on error
          return ""
        end try
      "#,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && std::path::Path::new(line).exists())
        .map(str::to_string)
        .collect();
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

pub fn frontmost_app() -> (Option<String>, Option<String>) {
    crate::app::platform::frontmost_app()
}
