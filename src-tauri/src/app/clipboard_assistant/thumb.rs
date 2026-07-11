use std::fs;
use std::io::Cursor;
use std::path::Path;

use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, RgbaImage};

use super::db::thumb_path;

const THUMB_MAX: u32 = 560;

/// 解码剪贴板图片：优先识别编码格式，否则按 arboard 原始 RGBA 像素解析。
pub fn decode_image(bytes: &[u8], dimensions: Option<(u32, u32)>) -> Result<DynamicImage, String> {
    if let Ok(img) = image::load_from_memory(bytes) {
        return Ok(img);
    }
    let Some((width, height)) = dimensions else {
        return Err("无法识别图片格式".into());
    };
    if width == 0 || height == 0 {
        return Err("无效图片尺寸".into());
    }
    let expected = (width as u64) * (height as u64) * 4;
    if bytes.len() as u64 != expected {
        return Err("RGBA 像素长度与尺寸不匹配".into());
    }
    let rgba = RgbaImage::from_raw(width, height, bytes.to_vec())
        .ok_or_else(|| "无法构造 RGBA 图像".to_string())?;
    Ok(DynamicImage::ImageRgba8(rgba))
}

/// 将剪贴板图片归一化为 WebP，便于持久化、缩略图与回写剪贴板。
pub fn normalize_image_bytes(
    bytes: Vec<u8>,
    dimensions: Option<(u32, u32)>,
) -> Result<Vec<u8>, String> {
    if image::load_from_memory(&bytes).is_ok() {
        return Ok(bytes);
    }
    let img = decode_image(&bytes, dimensions)?;
    encode_webp(&img)
}

/// 从图片字节生成 WebP 缩略图。
pub fn write_thumb_from_bytes(blobs_dir: &Path, id: i64, bytes: &[u8]) -> Result<(), String> {
    let img = decode_image(bytes, None)?;
    write_thumb_image(blobs_dir, id, &img)
}

/// 从磁盘文件生成缩略图（失败时静默跳过）。
pub fn write_thumb_from_path(blobs_dir: &Path, id: i64, path: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("读取文件失败：{e}"))?;
    write_thumb_from_bytes(blobs_dir, id, &bytes)
}

fn encode_webp(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP)
        .map_err(|e| format!("编码 WebP 失败：{e}"))?;
    Ok(buf)
}

fn write_thumb_image(
    blobs_dir: &Path,
    id: i64,
    img: &DynamicImage,
) -> Result<(), String> {
    let thumb = img.resize(THUMB_MAX, THUMB_MAX, FilterType::Triangle);
    let dest = thumb_path(blobs_dir, id);
    let mut buf = Vec::new();
    thumb
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP)
        .map_err(|e| format!("编码缩略图失败：{e}"))?;
    fs::write(dest, buf).map_err(|e| format!("写入缩略图失败：{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_raw_rgba_with_dimensions() {
        let width = 2u32;
        let height = 2u32;
        let bytes = vec![255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255];
        let img = decode_image(&bytes, Some((width, height))).expect("decode rgba");
        assert_eq!(img.width(), width);
        assert_eq!(img.height(), height);
    }

    #[test]
    fn normalizes_raw_rgba_to_webp() {
        let width = 2u32;
        let height = 2u32;
        let bytes = vec![255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255];
        let normalized = normalize_image_bytes(bytes, Some((width, height))).expect("normalize");
        assert!(image::load_from_memory(&normalized).is_ok());
    }
}
