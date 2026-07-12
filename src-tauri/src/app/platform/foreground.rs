//! 前台应用识别：供剪贴板来源、缩写展开跳过自身、焦点还原等共用。

/// 是否为 X11 会话（Linux 专用；Wayland 返回 false）。
#[cfg(all(unix, not(target_os = "macos")))]
pub fn is_x11_session() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_ok()
}

/// 返回 `(app_id, display_name)`。`app_id` 为平台相关稳定标识，用于缓存 key 与 DB 存储。
pub fn frontmost_app() -> (Option<String>, Option<String>) {
    #[cfg(target_os = "macos")]
    {
        macos_frontmost_app()
    }
    #[cfg(target_os = "windows")]
    {
        windows_frontmost_app()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if is_x11_session() {
            linux_x11_frontmost_app()
        } else {
            (None, None)
        }
    }
}

/// AppX 自身处于前台时返回 true（缩写展开应跳过）。
pub fn is_own_app_foreground(own_app_id: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_is_own_app_foreground(own_app_id)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = own_app_id;
        match (own_process_id(), foreground_process_id()) {
            (Some(own_pid), Some(fg_pid)) => own_pid == fg_pid,
            _ => false,
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_frontmost_app() -> (Option<String>, Option<String>) {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    let Some(app) = workspace.frontmostApplication() else {
        return (None, None);
    };
    let bundle = app.bundleIdentifier().map(|s| s.to_string());
    let name = app.localizedName().map(|s| s.to_string());
    (bundle, name)
}

#[cfg(target_os = "macos")]
fn macos_is_own_app_foreground(own_bundle_id: &str) -> bool {
    if own_bundle_id.is_empty() {
        return false;
    }
    let Some(bundle) = macos_frontmost_app().0 else {
        return false;
    };
    bundle == own_bundle_id
}

#[cfg(target_os = "windows")]
fn windows_frontmost_app() -> (Option<String>, Option<String>) {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;

    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return (None, None);
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return (None, None);
        }
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return (None, None);
        };
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        if !ok || len == 0 {
            return (None, None);
        }
        let path = OsString::from_wide(&buf[..len as usize]);
        let path_buf = PathBuf::from(path);
        let app_id = path_buf.to_string_lossy().to_ascii_lowercase();
        let name = path_buf
            .file_stem()
            .map(|s| s.to_string_lossy().to_string());
        (Some(app_id), name)
    }
}

#[cfg(target_os = "windows")]
fn own_process_id() -> Option<u32> {
    use windows::Win32::System::Threading::GetCurrentProcessId;
    Some(unsafe { GetCurrentProcessId() })
}

#[cfg(target_os = "windows")]
fn foreground_process_id() -> Option<u32> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 { None } else { Some(pid) }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_x11_frontmost_app() -> (Option<String>, Option<String>) {
    let Some((pid, wm_class)) = linux_x11_active_window_meta() else {
        return (None, None);
    };
    let app_id = format!("{wm_class}:{pid}");
    let name = linux_desktop_name_for_wm_class(&wm_class)
        .or_else(|| std::fs::read_to_string(format!("/proc/{pid}/comm")).ok());
    (Some(app_id), name)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn own_process_id() -> Option<u32> {
    Some(std::process::id())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn foreground_process_id() -> Option<u32> {
    linux_x11_active_window_meta().map(|(pid, _)| pid)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_x11_active_window_meta() -> Option<(u32, String)> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _};
    use x11rb::rust_connection::RustConnection;

    let (conn, screen_num) = RustConnection::connect(None).ok()?;
    let root = conn.setup().roots[screen_num].root;
    let active_atom = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let active = conn
        .get_property(
            false,
            root,
            active_atom,
            AtomEnum::WINDOW,
            0,
            1,
        )
        .ok()?
        .reply()
        .ok()?;
    let window = active.value32()?.next()?;
    if window == 0 {
        return None;
    }

    let pid_atom = conn
        .intern_atom(false, b"_NET_WM_PID")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let pid_reply = conn
        .get_property(
            false,
            window,
            pid_atom,
            AtomEnum::CARDINAL,
            0,
            1,
        )
        .ok()?
        .reply()
        .ok()?;
    let pid = pid_reply.value32()?.next().unwrap_or(0);
    if pid == 0 {
        return None;
    }

    let class_reply = conn
        .get_property(
            false,
            window,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            0,
            256,
        )
        .ok()?
        .reply()
        .ok()?;
    let wm_class = parse_wm_class(&class_reply.value).unwrap_or_else(|| format!("pid-{pid}"));
    Some((pid, wm_class))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn parse_wm_class(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?.trim_end_matches('\0');
    let mut parts = text.split('\0').filter(|s| !s.is_empty());
    let instance = parts.next()?;
    let class = parts.next().unwrap_or(instance);
    Some(format!("{instance}.{class}"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_desktop_name_for_wm_class(wm_class: &str) -> Option<String> {
    let class_lower = wm_class.to_ascii_lowercase();
    for dir in linux_desktop_dirs() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if !desktop_matches_wm_class(&content, &class_lower) {
                continue;
            }
            if let Some(name) = desktop_field(&content, "Name") {
                return Some(name);
            }
        }
    }
    None
}

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn linux_desktop_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = vec![
        std::path::PathBuf::from("/usr/share/applications"),
        std::path::PathBuf::from("/usr/local/share/applications"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(std::path::PathBuf::from(home).join(".local/share/applications"));
    }
    dirs
}

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn desktop_matches_wm_class(content: &str, wm_class: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("StartupWMClass=") {
            return value.trim().to_ascii_lowercase() == wm_class;
        }
    }
    false
}

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn desktop_field(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix(&prefix) {
            return Some(value.trim().to_string());
        }
    }
    None
}
