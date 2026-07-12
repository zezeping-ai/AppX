use arboard::Clipboard;

use crate::app::clipboard;

#[derive(Debug, Clone)]
pub struct ClipboardRead {
    pub text: Option<String>,
    pub image: Option<ImageCapture>,
    pub files: Option<Vec<String>>,
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

    Ok(ClipboardRead { text, image, files })
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
        hasher.update(&img.bytes[..img.bytes.len().min(4096)]);
        hasher.update(img.width.to_le_bytes());
        hasher.update(img.height.to_le_bytes());
    }
    if let Some(text) = &read.text {
        hasher.update(b"text:");
        hasher.update(text.as_bytes());
    }
    if read.files.is_none() && read.image.is_none() && read.text.is_none() {
        hasher.update(b"empty");
    }
    let digest = hasher.finalize();
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(target_os = "macos")]
fn read_macos_files() -> Option<Vec<String>> {
  use std::process::Command;

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
