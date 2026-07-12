//! 浮层窗口打开前记录前台应用，关闭后切回（命令面板、剪贴板历史等共用）。

use std::sync::Mutex;

static FOCUS_TARGET_APP_ID: Mutex<Option<String>> = Mutex::new(None);

/// 打开浮层前记录当前前台应用（跳过 AppX 自身）。
pub fn capture(own_app_id: &str) {
    let (app_id, _) = crate::app::platform::frontmost_app();
    let Some(app_id) = app_id else {
        return;
    };
    if crate::app::platform::is_own_app_foreground(own_app_id) {
        return;
    }
    if let Ok(mut guard) = FOCUS_TARGET_APP_ID.lock() {
        *guard = Some(app_id);
    }
}

/// 将焦点切回打开浮层前的应用。
pub fn restore() {
    #[cfg(target_os = "macos")]
    restore_macos();
    #[cfg(target_os = "windows")]
    restore_windows();
    #[cfg(all(unix, not(target_os = "macos")))]
    restore_linux_x11();
}

#[cfg(target_os = "macos")]
fn restore_macos() {
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
    use objc2_foundation::NSString;

    let app_id = FOCUS_TARGET_APP_ID
        .lock()
        .ok()
        .and_then(|guard| guard.clone());
    let Some(app_id) = app_id else {
        return;
    };

    let ns_bundle = NSString::from_str(&app_id);
    let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&ns_bundle);
    if apps.count() == 0 {
        return;
    }

    let app = apps.objectAtIndex(0);
    let _ = app.activateWithOptions(NSApplicationActivationOptions::empty());
}

#[cfg(target_os = "windows")]
fn restore_windows() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use windows::core::PWSTR;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, SetForegroundWindow,
    };

    let target = FOCUS_TARGET_APP_ID
        .lock()
        .ok()
        .and_then(|guard| guard.clone());
    let Some(target) = target else {
        return;
    };

    struct EnumState {
        target: String,
        found: HWND,
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let state = &mut *(lparam.0 as *mut EnumState);
            if !IsWindowVisible(hwnd).as_bool() {
                return BOOL(1);
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return BOOL(1);
            }
            let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
                return BOOL(1);
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
                return BOOL(1);
            }
            let path = OsString::from_wide(&buf[..len as usize])
                .to_string_lossy()
                .to_ascii_lowercase();
            if path == state.target {
                state.found = hwnd;
                return BOOL(0);
            }
            BOOL(1)
        }
    }

    let mut state = EnumState {
        target,
        found: HWND::default(),
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut state as *mut EnumState as isize),
        );
        if !state.found.0.is_null() {
            let _ = SetForegroundWindow(state.found);
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn restore_linux_x11() {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, PropMode};
    use x11rb::rust_connection::RustConnection;
    use x11rb::wrapper::ConnectionExt as _;

    let target = FOCUS_TARGET_APP_ID
        .lock()
        .ok()
        .and_then(|guard| guard.clone());
    let Some(target) = target else {
        return;
    };
    let target_pid = target
        .split(':')
        .nth(1)
        .and_then(|s| s.parse::<u32>().ok());
    let Some(target_pid) = target_pid else {
        return;
    };

    let Ok((conn, screen_num)) = RustConnection::connect(None) else {
        return;
    };
    let root = conn.setup().roots[screen_num].root;

    let Some(pid_atom) = conn
        .intern_atom(false, b"_NET_WM_PID")
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| r.atom)
    else {
        return;
    };

    let Some(tree) = conn
        .query_tree(root)
        .ok()
        .and_then(|c| c.reply().ok())
    else {
        return;
    };

    for &window in &tree.children {
        let Some(reply) = conn
            .get_property(
                false,
                window,
                pid_atom,
                AtomEnum::CARDINAL,
                0,
                1,
            )
            .ok()
            .and_then(|c| c.reply().ok())
        else {
            continue;
        };
        let Some(pid) = reply.value32().and_then(|mut it| it.next()) else {
            continue;
        };
        if pid != target_pid {
            continue;
        }

        let Some(active_atom) = conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| r.atom)
        else {
            break;
        };

        let _ = conn.change_property32(
            PropMode::REPLACE,
            root,
            active_atom,
            AtomEnum::WINDOW,
            &[window],
        );
        break;
    }
}
