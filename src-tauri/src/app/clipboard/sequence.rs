//! 系统剪贴板变更令牌：用于廉价轮询，避免每次全量读取。

/// 返回当前剪贴板变更令牌；平台不支持时返回 `None`（回退指纹比对）。
pub fn change_token() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        return macos::change_token();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::change_token();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use objc2_app_kit::NSPasteboard;

    pub fn change_token() -> Option<u64> {
        crate::app::clipboard::with_pasteboard_lock(|| {
            let pasteboard = NSPasteboard::generalPasteboard();
            Some(pasteboard.changeCount() as u64)
        })
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;

    pub fn change_token() -> Option<u64> {
        // SAFETY: Win32 文档保证该 API 无额外前置条件，返回全局剪贴板序列号。
        Some(unsafe { GetClipboardSequenceNumber() } as u64)
    }
}
