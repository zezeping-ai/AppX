use arboard::{Clipboard, ImageData};

use super::{with_pasteboard_lock, with_record_suppressed};

pub fn write_image_bytes(bytes: &[u8]) -> Result<(), String> {
    with_pasteboard_lock(|| {
        with_record_suppressed(|| {
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
