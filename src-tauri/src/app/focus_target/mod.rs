//! 浮层打开前记录焦点目标，关闭后统一还原（命令面板、剪切面板共用）。
//!
//! 调用约定：
//! - 打开前：`capture`
//! - 粘贴 / Esc 等主动关闭：`restore`（把焦点还回去）
//! - 点外关闭等被动解散：`discard`（用户已点到别处，不抢焦点）

use std::sync::Mutex;

use tauri::{AppHandle, Manager};

const MAIN_WINDOW_LABEL: &str = "main";

struct CapturedFocus {
    /// 平台应用标识（bundle id / 可执行路径 / wm_class:pid）
    app_id: String,
    /// AppX 自身：面板 make_key 会抢走 WebView 编辑焦点，还需还原 DOM 节点
    web_editable: bool,
}

static TARGET: Mutex<Option<CapturedFocus>> = Mutex::new(None);

/// 打开浮层前记录当前前台目标；每次都会清掉上次残留。
pub fn capture(app: &AppHandle, own_app_id: &str) {
    discard();

    let own = crate::app::platform::is_own_app_foreground(own_app_id);
    let app_id = if own {
        own_app_id.to_string()
    } else {
        let Some(id) = crate::app::platform::frontmost_app().0 else {
            return;
        };
        id
    };

    if own {
        capture_own_web_focus(app);
    }

    if let Ok(mut guard) = TARGET.lock() {
        *guard = Some(CapturedFocus {
            app_id,
            web_editable: own,
        });
    }
}

/// 还原到 `capture` 时的目标（取走后清空，可重复调用）。
pub fn restore(app: &AppHandle) {
    let Some(target) = take() else {
        return;
    };
    if target.web_editable {
        restore_own_web_focus(app);
        return;
    }
    activate_app(&target.app_id);
}

/// 放弃还原，只清状态（点外关闭、会话锁定等）。
pub fn discard() {
    let _ = take();
}

fn take() -> Option<CapturedFocus> {
    TARGET.lock().ok().and_then(|mut guard| guard.take())
}

/// 在面板抢 key 之前记下可编辑节点，供粘贴前还原。
fn capture_own_web_focus(app: &AppHandle) {
    let Some(main) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
    };
    let _ = main.eval(
        r#"(function () {
  try {
    var a = document.activeElement;
    if (!a || a === document.body || a === document.documentElement) {
      window.__appxPasteTarget = null;
      return;
    }
    if (!(a.isContentEditable || a.tagName === 'INPUT' || a.tagName === 'TEXTAREA' || a.tagName === 'SELECT')) {
      var host = a.closest && a.closest('[contenteditable="true"], [contenteditable=""]');
      if (!host) {
        window.__appxPasteTarget = null;
        return;
      }
      a = host;
    }
    window.__appxPasteTarget = a;
  } catch (_) {
    window.__appxPasteTarget = null;
  }
})();"#,
    );
}

fn restore_own_web_focus(app: &AppHandle) {
    let Some(main) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
    };
    if main.is_visible().unwrap_or(false) {
        let _ = main.set_focus();
    }
    let _ = main.eval(
        r#"(function () {
  try {
    var a = window.__appxPasteTarget;
    window.__appxPasteTarget = null;
    if (!a || !document.contains(a)) return;
    a.focus({ preventScroll: true });
  } catch (_) {}
})();"#,
    );
}

fn activate_app(app_id: &str) {
    #[cfg(target_os = "macos")]
    activate_macos(app_id);
    #[cfg(target_os = "windows")]
    activate_windows(app_id);
    #[cfg(all(unix, not(target_os = "macos")))]
    activate_linux_x11(app_id);
}

#[cfg(target_os = "macos")]
fn activate_macos(app_id: &str) {
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
    use objc2_foundation::NSString;

    let ns_bundle = NSString::from_str(app_id);
    let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&ns_bundle);
    if apps.count() == 0 {
        return;
    }
    let app = apps.objectAtIndex(0);
    let _ = app.activateWithOptions(NSApplicationActivationOptions::empty());
}

#[cfg(target_os = "windows")]
fn activate_windows(target: &str) {
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
        target: target.to_string(),
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
fn activate_linux_x11(target: &str) {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, PropMode};
    use x11rb::rust_connection::RustConnection;
    use x11rb::wrapper::ConnectionExt as _;

    let Some(target_pid) = target
        .split(':')
        .nth(1)
        .and_then(|s| s.parse::<u32>().ok())
    else {
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
            .get_property(false, window, pid_atom, AtomEnum::CARDINAL, 0, 1)
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
