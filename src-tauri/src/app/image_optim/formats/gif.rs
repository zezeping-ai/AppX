use std::io::Cursor;

use gif::{Encoder, Frame, Repeat};

use super::quant::neuquant_params;
use super::{finish_candidates, OptimizeOptions};

pub fn optimize_gif(data: &[u8], opts: &OptimizeOptions) -> Result<Option<Vec<u8>>, String> {
    let mut candidates = Vec::new();
    let mut last_err: Option<String> = None;

    match reencode_indexed(data) {
        Ok(bytes) => candidates.push(bytes),
        Err(err) if !opts.lossy => return Err(err),
        Err(err) => last_err = Some(err),
    }

    if opts.lossy && opts.gif_quality < 100 {
        match reencode_lossy(data, opts.gif_quality) {
            Ok(bytes) => candidates.push(bytes),
            Err(err) => last_err = Some(err),
        }
    }

    finish_candidates(data, candidates, last_err)
}

struct IndexedFrame {
    delay: u16,
    dispose: gif::DisposalMethod,
    transparent: Option<u8>,
    needs_user_input: bool,
    top: u16,
    left: u16,
    width: u16,
    height: u16,
    palette: Option<Vec<u8>>,
    indices: Vec<u8>,
}

fn reencode_indexed(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    let mut reader = decoder
        .read_info(Cursor::new(data))
        .map_err(|e| format!("解码 GIF 失败：{e}"))?;

    let width = reader.width();
    let height = reader.height();
    let global_palette = reader.global_palette().map(|p| p.to_vec()).unwrap_or_default();
    let repeat = reader.repeat();

    let mut frames = Vec::new();
    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|e| format!("读取 GIF 帧失败：{e}"))?
    {
        frames.push(IndexedFrame {
            delay: frame.delay,
            dispose: frame.dispose,
            transparent: frame.transparent,
            needs_user_input: frame.needs_user_input,
            top: frame.top,
            left: frame.left,
            width: frame.width,
            height: frame.height,
            palette: frame.palette.clone(),
            indices: frame.buffer.to_vec(),
        });
    }
    if frames.is_empty() {
        return Err("GIF 无有效帧".into());
    }

    write_indexed_gif(width, height, &global_palette, repeat, &frames)
}

fn reencode_lossy(data: &[u8], quality: u8) -> Result<Vec<u8>, String> {
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = decoder
        .read_info(Cursor::new(data))
        .map_err(|e| format!("解码 GIF 失败：{e}"))?;

    let width = reader.width();
    let height = reader.height();
    let repeat = reader.repeat();
    let (sample_fac, ncolors) = neuquant_params(quality);

    let mut out = Vec::new();
    {
        let mut encoder = Encoder::new(&mut out, width, height, &[])
            .map_err(|e| format!("创建 GIF 编码器失败：{e}"))?;
        encoder
            .set_repeat(repeat)
            .map_err(|e| format!("设置 GIF 循环失败：{e}"))?;

        while let Some(frame) = reader
            .read_next_frame()
            .map_err(|e| format!("读取 GIF 帧失败：{e}"))?
        {
            let mut rgba = frame.buffer.to_vec();
            let nq = color_quant::NeuQuant::new(sample_fac, ncolors, &rgba);
            let map = nq.color_map_rgb();
            let mut indices = Vec::with_capacity(rgba.len() / 4);
            let mut transparent: Option<u8> = None;
            for pix in rgba.chunks_exact_mut(4) {
                let idx = nq.index_of(pix) as u8;
                if pix[3] < 128 && transparent.is_none() {
                    transparent = Some(idx);
                }
                indices.push(idx);
            }
            let mut encoded = Frame::from_palette_pixels(
                frame.width,
                frame.height,
                indices,
                map,
                transparent,
            );
            encoded.delay = frame.delay;
            encoded.dispose = frame.dispose;
            encoded.needs_user_input = frame.needs_user_input;
            encoded.top = frame.top;
            encoded.left = frame.left;
            encoder
                .write_frame(&encoded)
                .map_err(|e| format!("写入 GIF 帧失败：{e}"))?;
        }
    }
    Ok(out)
}

fn write_indexed_gif(
    width: u16,
    height: u16,
    global_palette: &[u8],
    repeat: Repeat,
    frames: &[IndexedFrame],
) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let mut encoder = Encoder::new(&mut out, width, height, global_palette)
            .map_err(|e| format!("创建 GIF 编码器失败：{e}"))?;
        encoder
            .set_repeat(repeat)
            .map_err(|e| format!("设置 GIF 循环失败：{e}"))?;

        for frame in frames {
            let mut encoded = if let Some(ref palette) = frame.palette {
                Frame::from_palette_pixels(
                    frame.width,
                    frame.height,
                    frame.indices.clone(),
                    palette.clone(),
                    frame.transparent,
                )
            } else {
                Frame::from_indexed_pixels(
                    frame.width,
                    frame.height,
                    frame.indices.clone(),
                    frame.transparent,
                )
            };
            encoded.delay = frame.delay;
            encoded.dispose = frame.dispose;
            encoded.needs_user_input = frame.needs_user_input;
            encoded.top = frame.top;
            encoded.left = frame.left;
            encoder
                .write_frame(&encoded)
                .map_err(|e| format!("写入 GIF 帧失败：{e}"))?;
        }
    }
    Ok(out)
}
