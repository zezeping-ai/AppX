//! 系统剪贴板唯一入口；后续剪贴板历史在此扩展。

use arboard::Clipboard;

/// 用户主动拷贝：写入剪贴板并保留（未来可记录历史）。
pub fn set_text_persist(text: &str) -> Result<(), String> {
    Clipboard::new()
        .map_err(|err| format!("无法访问剪贴板：{err}"))?
        .set_text(text.to_string())
        .map_err(|err| format!("写入剪贴板失败：{err}"))
}

/// 插入时的临时写入：返回原剪贴板内容供恢复。
pub fn set_text_transient(text: &str) -> Result<Option<String>, String> {
    let mut clipboard = Clipboard::new().map_err(|err| format!("无法访问剪贴板：{err}"))?;
    let original = clipboard.get_text().ok();
    clipboard
        .set_text(text.to_string())
        .map_err(|err| format!("写入剪贴板失败：{err}"))?;
    Ok(original)
}

/// 恢复剪贴板内容（插入副作用结束后调用）。
pub fn restore_text(text: &str) {
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(text.to_string());
    }
}
