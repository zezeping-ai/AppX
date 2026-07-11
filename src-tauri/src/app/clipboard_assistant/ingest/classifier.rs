use super::reader::ClipboardRead;
use super::super::model::CapturedPayload;
use super::super::model::PayloadKind;

pub fn classify(read: ClipboardRead) -> Option<CapturedPayload> {
    let text = read.text.filter(|t| !t.trim().is_empty());
    let files = read.files.filter(|f| !f.is_empty());

    let prefer_text = match (&text, &files) {
        (Some(value), Some(paths)) => !text_is_redundant_with_files(value, paths),
        (Some(_), None) => true,
        _ => false,
    };

    if prefer_text {
        return Some(CapturedPayload {
            kind: PayloadKind::Inline,
            text,
            file_paths: None,
            image_bytes: None,
            image_dimensions: None,
        });
    }

    if let Some(files) = files {
        return Some(CapturedPayload {
            kind: PayloadKind::FileRef,
            text: None,
            file_paths: Some(files),
            image_bytes: None,
            image_dimensions: None,
        });
    }

    if let Some(image) = read.image {
        return Some(CapturedPayload {
            kind: PayloadKind::Blob,
            text: None,
            file_paths: None,
            image_bytes: Some(image.bytes),
            image_dimensions: Some((image.width, image.height)),
        });
    }

    if let Some(text) = text {
        return Some(CapturedPayload {
            kind: PayloadKind::Inline,
            text: Some(text),
            file_paths: None,
            image_bytes: None,
            image_dimensions: None,
        });
    }

    None
}

/// 剪贴板同时含文件引用与文本时，判断文本是否只是路径/文件名（应视为复制文件）。
fn text_is_redundant_with_files(text: &str, files: &[String]) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    // 多行或较长内容视为真实文本，而非文件路径别名。
    if trimmed.contains('\n') || trimmed.chars().count() > 512 {
        return false;
    }
    files.iter().any(|path| text_matches_file_path(trimmed, path))
}

fn text_matches_file_path(text: &str, path: &str) -> bool {
    if text == path {
        return true;
    }
    if let Some(name) = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
    {
        if text == name {
            return true;
        }
    }
    if let Some(stripped) = text.strip_prefix("file://") {
        if stripped == path {
            return true;
        }
        if let Some(name) = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
        {
            if stripped == name {
                return true;
            }
        }
    }
    false
}

pub fn maybe_compress_image(
    bytes: Vec<u8>,
    max_soft: u32,
    max_hard: u32,
    compress: bool,
) -> Option<Vec<u8>> {
    let len = bytes.len() as u32;
    if len <= max_soft {
        return Some(bytes);
    }
    if !compress || len > max_hard {
        return None;
    }
    let Ok(img) = image::load_from_memory(&bytes) else {
        return None;
    };
    let mut buf = Vec::new();
    let scaled = img.resize(2048, 2048, image::imageops::FilterType::Triangle);
    if scaled
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::WebP)
        .is_err()
    {
        return None;
    }
    if buf.len() as u32 > max_hard {
        return None;
    }
    Some(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_text_when_not_file_path() {
        let read = ClipboardRead {
            text: Some("hello from document".into()),
            image: None,
            files: Some(vec!["/Users/me/doc.txt".into()]),
        };
        let payload = classify(read).unwrap();
        assert_eq!(payload.kind, PayloadKind::Inline);
        assert_eq!(payload.text.as_deref(), Some("hello from document"));
    }

    #[test]
    fn prefers_file_when_text_is_path() {
        let path = "/Users/me/report.pdf";
        let read = ClipboardRead {
            text: Some(path.into()),
            image: None,
            files: Some(vec![path.into()]),
        };
        let payload = classify(read).unwrap();
        assert_eq!(payload.kind, PayloadKind::FileRef);
    }

    #[test]
    fn prefers_file_when_text_is_filename() {
        let read = ClipboardRead {
            text: Some("report.pdf".into()),
            image: None,
            files: Some(vec!["/Users/me/report.pdf".into()]),
        };
        let payload = classify(read).unwrap();
        assert_eq!(payload.kind, PayloadKind::FileRef);
    }
}
