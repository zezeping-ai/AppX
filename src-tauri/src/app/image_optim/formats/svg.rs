use oxvg_ast::{
    parse::roxmltree::parse,
    serialize::Node as _,
    visitor::Info,
};
use oxvg_optimiser::Jobs;

use super::{finish_candidates, OptimizeOptions};

const MAX_SVG_BYTES: usize = 16 * 1024 * 1024;

pub fn optimize_svg(data: &[u8], opts: &OptimizeOptions) -> Result<Option<Vec<u8>>, String> {
    if data.len() > MAX_SVG_BYTES {
        return Err("SVG 过大（上限 16MB）".into());
    }
    let input = std::str::from_utf8(data).map_err(|_| "SVG 不是合法 UTF-8".to_string())?;

    let mut candidates = Vec::new();
    let mut last_err: Option<String> = None;

    match run_jobs(input, Jobs::safe()) {
        Ok(bytes) => candidates.push(bytes),
        Err(err) if !(opts.lossy || opts.effort >= 5) => return Err(err),
        Err(err) => last_err = Some(err),
    }

    // 有损或高强度：default 预设更激进（oxvg 已清理注释等元数据）
    if opts.lossy || opts.effort >= 5 {
        match run_jobs(input, Jobs::default()) {
            Ok(bytes) => candidates.push(bytes),
            Err(err) => last_err = Some(err),
        }
    }

    finish_candidates(data, candidates, last_err)
}

fn run_jobs(input: &str, jobs: Jobs) -> Result<Vec<u8>, String> {
    parse(input, |dom, allocator| {
        jobs.run(dom, &Info::new(allocator))
            .map_err(|e| format!("SVG 优化失败：{e}"))?;
        dom.serialize()
            .map_err(|e| format!("SVG 序列化失败：{e}"))
    })
    .map_err(|e| format!("解析 SVG 失败：{e}"))
    .and_then(|inner| inner)
    .map(|s| s.into_bytes())
}
