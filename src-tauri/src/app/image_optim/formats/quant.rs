/// NeuQuant 采样因子与色数（质量越高色越多、采样越细）
pub fn neuquant_params(quality: u8) -> (i32, usize) {
    let sample_fac = ((100 - u32::from(quality)) / 5 + 1).clamp(1, 30) as i32;
    let ncolors = match quality {
        90..=99 => 256,
        80..=89 => 192,
        70..=79 => 128,
        60..=69 => 96,
        _ => 64,
    };
    (sample_fac, ncolors)
}
