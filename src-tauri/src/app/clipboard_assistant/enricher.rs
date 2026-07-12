use sha2::{Digest, Sha256};

use super::model::{ContentBadge, ContentType};

const PREVIEW_MAX: usize = 320;

pub struct EnrichedMeta {
    pub preview: String,
    pub content_type: ContentType,
    pub group_key: String,
    pub accent_color: String,
    pub badges: Vec<ContentBadge>,
    pub char_count: Option<i64>,
    pub content_hash: String,
}

pub fn hash_bytes(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

#[allow(dead_code)]
pub fn hash_text(text: &str) -> String {
    hash_bytes(text.as_bytes())
}

/// 内容指纹：纯文本 + 可选富文本摘要（与监听 fingerprint 语义对齐）。
pub fn hash_content(text: &str, rich: Option<&crate::app::clipboard::rich::RichFormats>) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"text:");
    hasher.update(text.as_bytes());
    if let Some(rich) = rich.filter(|value| value.has_content()) {
        if let Some(html) = &rich.html {
            hasher.update(b"html:");
            hasher.update(html.as_bytes());
        }
        if let Some(rtf) = &rich.rtf {
            hasher.update(b"rtf:");
            hasher.update(&rtf[..rtf.len().min(4096)]);
        }
    }
    let digest = hasher.finalize();
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

/// 从 HTML 提取可读纯文本（用于 preview / 富文本-only 入库）。
pub fn plain_text_from_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn hash_paths(paths: &[String]) -> String {
    hash_bytes(paths.join("\n").as_bytes())
}

pub fn make_preview(text: &str) -> String {
    let normalized = text.replace('\r', "");
    let normalized = normalized.trim();
    if normalized.is_empty() {
        return String::new();
    }
    if normalized.chars().count() <= PREVIEW_MAX {
        return normalized.to_string();
    }
    normalized.chars().take(PREVIEW_MAX).collect()
}

pub fn enrich_text(
    text: &str,
    bundle: Option<&str>,
    rich: Option<&crate::app::clipboard::rich::RichFormats>,
) -> EnrichedMeta {
    let trimmed = text.trim();
    let content_type = detect_text_type(trimmed);
    let group_key = group_for_type(content_type);
    let mut badges = vec![];
    if content_type == ContentType::Link {
        badges.push(ContentBadge {
            kind: "url".into(),
            label: "链接".into(),
        });
    }
    if looks_like_json(trimmed) {
        badges.push(ContentBadge {
            kind: "json".into(),
            label: "JSON".into(),
        });
    }
    if looks_like_code(trimmed) {
        badges.push(ContentBadge {
            kind: "code".into(),
            label: "代码".into(),
        });
    }
    if looks_like_color(trimmed) {
        badges.push(ContentBadge {
            kind: "color".into(),
            label: "颜色".into(),
        });
    }
    if rich.is_some_and(|value| value.has_content()) {
        badges.push(ContentBadge {
            kind: "rich".into(),
            label: "富文本".into(),
        });
    }
    EnrichedMeta {
        preview: make_preview(trimmed),
        content_type,
        group_key,
        accent_color: accent_color(content_type, bundle),
        badges,
        char_count: Some(trimmed.chars().count() as i64),
        content_hash: hash_content(trimmed, rich),
    }
}

pub fn enrich_file(paths: &[String], bundle: Option<&str>) -> EnrichedMeta {
    let name = paths
        .first()
        .and_then(|p| std::path::Path::new(p).file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("文件");
    let preview = if paths.len() > 1 {
        format!("{} 等 {} 个文件", name, paths.len())
    } else {
        name.to_string()
    };
    let content_type = if is_image_path(name) {
        ContentType::Image
    } else {
        ContentType::File
    };
    EnrichedMeta {
        preview,
        content_type,
        group_key: group_for_type(content_type),
        accent_color: accent_color(content_type, bundle),
        badges: vec![ContentBadge {
            kind: "file".into(),
            label: "文件".into(),
        }],
        char_count: None,
        content_hash: hash_paths(paths),
    }
}

pub fn enrich_image(bundle: Option<&str>) -> EnrichedMeta {
    EnrichedMeta {
        preview: "图片".into(),
        content_type: ContentType::Image,
        group_key: "image".into(),
        accent_color: accent_color(ContentType::Image, bundle),
        badges: vec![ContentBadge {
            kind: "image".into(),
            label: "图片".into(),
        }],
        char_count: None,
        content_hash: String::new(),
    }
}

fn detect_text_type(text: &str) -> ContentType {
    if text.starts_with("http://") || text.starts_with("https://") {
        return ContentType::Link;
    }
    if looks_like_json(text) {
        return ContentType::Json;
    }
    if looks_like_code(text) {
        return ContentType::Code;
    }
    if looks_like_color(text) {
        return ContentType::Color;
    }
    ContentType::Text
}

fn group_for_type(t: ContentType) -> String {
    match t {
        ContentType::Link => "url".into(),
        ContentType::Image => "image".into(),
        ContentType::File => "file".into(),
        ContentType::Code => "code".into(),
        ContentType::Json => "json".into(),
        ContentType::Color => "color".into(),
        ContentType::Text => "default".into(),
    }
}

fn accent_color(t: ContentType, bundle: Option<&str>) -> String {
    let _base = match t {
        ContentType::Link => "#22c55e",
        ContentType::Image => "#a855f7",
        ContentType::File => "#f97316",
        ContentType::Code => "#3b82f6",
        ContentType::Json => "#06b6d4",
        ContentType::Color => "#ec4899",
        ContentType::Text => "#6b7280",
    };
    if let Some(bundle) = bundle.filter(|b| !b.is_empty()) {
        let h = hash_bytes(bundle.as_bytes());
        let n = u32::from_str_radix(&h[..4], 16).unwrap_or(0);
        let palette = ["#ef4444", "#f59e0b", "#10b981", "#6366f1", "#8b5cf6"];
        return palette[(n as usize) % palette.len()].to_string();
    }
    _base.to_string()
}

fn looks_like_json(s: &str) -> bool {
    let t = s.trim();
    (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']'))
}

fn looks_like_code(s: &str) -> bool {
    s.contains("function ") || s.contains("const ") || s.contains("=>") || s.contains("class ")
}

fn looks_like_color(s: &str) -> bool {
    normalize_color(s).is_some()
}

fn normalize_color(s: &str) -> Option<String> {
    let t = s.trim();
    if t.starts_with('#') {
        return normalize_hex_color(t);
    }
    None
}

fn normalize_hex_color(s: &str) -> Option<String> {
    let hex = s.trim().trim_start_matches('#');
    match hex.len() {
        3 if hex.chars().all(|c| c.is_ascii_hexdigit()) => {
            let expanded: String = hex.chars().flat_map(|c| [c, c]).collect();
            Some(format!("#{}", expanded.to_ascii_lowercase()))
        }
        6 | 8 if hex.chars().all(|c| c.is_ascii_hexdigit()) => {
            Some(format!("#{}", hex.to_ascii_lowercase()))
        }
        _ => None,
    }
}

fn is_image_path(name: &str) -> bool {
    let lower = name.to_lowercase();
    ["png", "jpg", "jpeg", "gif", "webp", "bmp", "heic"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::{make_preview, normalize_color, plain_text_from_html};

    #[test]
    fn strips_html_for_plain_preview() {
        assert_eq!(
            plain_text_from_html("<p><b>Hello</b> world</p>"),
            "Hello world"
        );
    }

    #[test]
    fn preview_keeps_multiple_lines() {
        let text = "标题行\n第二段内容\n第三段内容";
        assert_eq!(make_preview(text), text);
    }

    #[test]
    fn preview_truncates_by_chars_not_lines() {
        let text = "a\n".repeat(400);
        let preview = make_preview(&text);
        assert!(preview.chars().count() <= 320);
    }

    #[test]
    fn normalizes_short_hex_color() {
        assert_eq!(normalize_color("#f00").as_deref(), Some("#ff0000"));
    }

    #[test]
    fn normalizes_full_hex_color() {
        assert_eq!(normalize_color("#A855F7").as_deref(), Some("#a855f7"));
    }
}
