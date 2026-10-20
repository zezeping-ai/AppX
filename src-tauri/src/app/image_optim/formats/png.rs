use image::ImageEncoder;
use oxipng::{Deflater, Options as OxipngOptions, StripChunks, ZopfliOptions};

use super::quant::neuquant_params;
use super::{pick_smallest, OptimizeOptions};

pub fn optimize_png(data: &[u8], opts: &OptimizeOptions) -> Result<Option<Vec<u8>>, String> {
    let mut candidates = Vec::new();
    candidates.push(oxipng_optimize(data, opts)?);

    if opts.lossy && opts.png_min_quality < 100 {
        if let Ok(lossy) = lossy_quantize_png(data, opts) {
            match oxipng_optimize(&lossy, opts) {
                Ok(finished) => candidates.push(finished),
                Err(_) => candidates.push(lossy),
            }
        }
    }

    Ok(pick_smallest(data, candidates))
}

fn oxipng_optimize(data: &[u8], opts: &OptimizeOptions) -> Result<Vec<u8>, String> {
    let mut oxi = OxipngOptions::from_preset(opts.effort.min(6));
    if opts.strip_metadata {
        oxi.strip = StripChunks::Safe;
    }
    if opts.effort >= 6 {
        oxi.deflater = Deflater::Zopfli(ZopfliOptions::default());
    }
    oxipng::optimize_from_memory(data, &oxi).map_err(|e| format!("PNG 优化失败：{e}"))
}

fn lossy_quantize_png(data: &[u8], opts: &OptimizeOptions) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(data).map_err(|e| format!("解码 PNG 失败：{e}"))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels = rgba.into_raw();

    let (sample_fac, ncolors) = neuquant_params(opts.png_min_quality);
    let nq = color_quant::NeuQuant::new(sample_fac, ncolors, &pixels);
    let map = nq.color_map_rgba();
    let mut rebuilt = Vec::with_capacity(pixels.len());
    for pix in pixels.chunks_exact(4) {
        let idx = nq.index_of(pix) as usize * 4;
        if idx + 3 < map.len() {
            rebuilt.extend_from_slice(&map[idx..idx + 4]);
        } else {
            rebuilt.extend_from_slice(pix);
        }
    }

    let mut png_buf = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_buf)
        .write_image(&rebuilt, w, h, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("编码有损 PNG 失败：{e}"))?;
    Ok(png_buf)
}
