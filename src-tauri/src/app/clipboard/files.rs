pub fn write_file_paths(paths: &[String]) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return macos::write_file_paths(paths);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let joined = paths.join("\n");
        super::set_text_persist(&joined)
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::process::Command;

    use super::super::{with_pasteboard_lock, with_record_suppressed};

    pub fn write_file_paths(paths: &[String]) -> Result<(), String> {
        with_pasteboard_lock(|| {
            with_record_suppressed(|| {
                let quoted: Vec<String> = paths
                    .iter()
                    .map(|p| format!("POSIX file {:?}", p))
                    .collect();
                let script = if quoted.len() == 1 {
                    format!("set the clipboard to {}", quoted[0])
                } else {
                    format!("set the clipboard to {{{}}}", quoted.join(", "))
                };
                let output = Command::new("osascript")
                    .args(["-e", &script])
                    .output()
                    .map_err(|e| format!("调用 osascript 失败：{e}"))?;
                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("写入文件剪贴板失败：{err}"));
                }
                Ok(())
            })
        })
    }
}
