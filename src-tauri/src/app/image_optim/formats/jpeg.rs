use super::{pick_smallest, OptimizeOptions};

pub fn optimize_jpeg(data: &[u8], opts: &OptimizeOptions) -> Result<Option<Vec<u8>>, String> {
    let mut candidates = Vec::new();

    if opts.strip_metadata {
        if let Some(stripped) = strip_jpeg_markers(data) {
            candidates.push(stripped);
        }
    }

    if opts.lossy && opts.jpeg_max_quality < 100 {
        match lossy_reencode(data, opts.jpeg_max_quality) {
            Ok(bytes) => candidates.push(bytes),
            Err(err) if candidates.is_empty() => return Err(err),
            Err(_) => {}
        }
    }

    Ok(pick_smallest(data, candidates))
}

/// 剥离 APP1–APP15 / COM；解析失败或结果无效时返回 None（绝不交出截断文件）
fn strip_jpeg_markers(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&[0xFF, 0xD8]);
    let mut i = 2;
    let mut saw_eoi = false;
    // CMYK / YCCK 依赖 APP14；剥离时需保留
    let mut keep_app14 = false;

    // 预扫 SOF 判断是否可能为 CMYK（components == 4）
    if let Some(components) = peek_sof_components(data) {
        keep_app14 = components >= 4;
    }

    while i + 1 < data.len() {
        if data[i] != 0xFF {
            return None;
        }
        while i < data.len() && data[i] == 0xFF {
            i += 1;
        }
        if i >= data.len() {
            return None;
        }
        let marker = data[i];
        i += 1;

        if marker == 0xD9 {
            out.extend_from_slice(&[0xFF, 0xD9]);
            saw_eoi = true;
            break;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            out.push(0xFF);
            out.push(marker);
            continue;
        }

        if marker == 0xDA {
            if i + 1 >= data.len() {
                return None;
            }
            let len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
            if len < 2 || i + len > data.len() {
                return None;
            }
            out.push(0xFF);
            out.push(0xDA);
            out.extend_from_slice(&data[i..i + len]);
            i += len;

            let start = i;
            while i + 1 < data.len() {
                if data[i] == 0xFF {
                    let m = data[i + 1];
                    if m == 0x00 {
                        i += 2;
                        continue;
                    }
                    if m == 0xFF {
                        i += 1;
                        continue;
                    }
                    if m == 0xD9 {
                        out.extend_from_slice(&data[start..=i + 1]);
                        return finalize_strip(data, out);
                    }
                    if (0xD0..=0xD7).contains(&m) {
                        i += 2;
                        continue;
                    }
                    // 非法熵编码区 marker
                    return None;
                }
                i += 1;
            }
            return None;
        }

        if i + 1 >= data.len() {
            return None;
        }
        let len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
        if len < 2 || i + len > data.len() {
            return None;
        }

        let strip = match marker {
            0xFE => true,                         // COM
            0xE1..=0xEF if marker != 0xEE => true, // APP1–APP13, APP15
            0xEE => !keep_app14,                  // APP14 Adobe
            _ => false,
        };
        if !strip {
            out.push(0xFF);
            out.push(marker);
            out.extend_from_slice(&data[i..i + len]);
        }
        i += len;
    }

    if !saw_eoi {
        return None;
    }
    finalize_strip(data, out)
}

fn finalize_strip(original: &[u8], out: Vec<u8>) -> Option<Vec<u8>> {
    if out.len() < 4 || out.len() >= original.len() {
        return None;
    }
    if out[0] != 0xFF || out[1] != 0xD8 || out[out.len() - 2] != 0xFF || out[out.len() - 1] != 0xD9
    {
        return None;
    }
    Some(out)
}

fn peek_sof_components(data: &[u8]) -> Option<u8> {
    let mut i = 2;
    while i + 3 < data.len() {
        if data[i] != 0xFF {
            return None;
        }
        while i < data.len() && data[i] == 0xFF {
            i += 1;
        }
        if i >= data.len() {
            return None;
        }
        let marker = data[i];
        i += 1;
        if marker == 0xDA || marker == 0xD9 {
            return None;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }
        if i + 1 >= data.len() {
            return None;
        }
        let len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
        if len < 2 || i + len > data.len() {
            return None;
        }
        // SOF0–SOF3, SOF5–SOF7, SOF9–SOF11, SOF13–SOF15
        if matches!(
            marker,
            0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB | 0xCD | 0xCE
                | 0xCF
        ) && len >= 8
        {
            // SOF: Lf(2) P(1) Y(2) X(2) Nf(1) ...
            return Some(data[i + 7]);
        }
        i += len;
    }
    None
}

fn lossy_reencode(data: &[u8], quality: u8) -> Result<Vec<u8>, String> {
    let result = std::panic::catch_unwind(|| -> Result<Vec<u8>, String> {
        let img = image::load_from_memory(data).map_err(|e| format!("解码 JPEG 失败：{e}"))?;
        let rgb = img.to_rgb8();
        let (w, h) = rgb.dimensions();

        let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        comp.set_size(w as usize, h as usize);
        comp.set_quality(f32::from(quality));
        comp.set_optimize_coding(true);
        comp.set_progressive_mode();
        let mut comp = comp
            .start_compress(Vec::new())
            .map_err(|e| format!("JPEG 编码启动失败：{e}"))?;
        comp.write_scanlines(rgb.as_raw())
            .map_err(|e| format!("JPEG 写入失败：{e}"))?;
        comp.finish()
            .map_err(|e| format!("JPEG 完成失败：{e}"))
    });

    match result {
        Ok(inner) => inner,
        Err(_) => Err("JPEG 有损编码失败（内部错误）".into()),
    }
}
