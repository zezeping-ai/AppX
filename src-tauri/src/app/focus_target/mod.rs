//! 浮层窗口打开前记录前台应用，关闭后切回（命令面板、剪贴板历史等共用）。

use std::sync::Mutex;

static FOCUS_TARGET_BUNDLE: Mutex<Option<String>> = Mutex::new(None);

/// 打开浮层前记录当前前台应用（跳过 AppX 自身）。
pub fn capture(own_bundle_id: &str) {
    let Some(bundle_id) = frontmost_bundle_id() else {
        return;
    };
    if bundle_id == own_bundle_id {
        return;
    }
    if let Ok(mut guard) = FOCUS_TARGET_BUNDLE.lock() {
        *guard = Some(bundle_id);
    }
}

/// 将焦点切回打开浮层前的应用。
pub fn restore() {
    #[cfg(target_os = "macos")]
    {
        restore_macos();
    }
}

#[cfg(target_os = "macos")]
fn frontmost_bundle_id() -> Option<String> {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.frontmostApplication()?;
    let bundle = app.bundleIdentifier()?;
    Some(bundle.to_string())
}

#[cfg(not(target_os = "macos"))]
fn frontmost_bundle_id() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
fn restore_macos() {
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
    use objc2_foundation::NSString;

    let bundle_id = FOCUS_TARGET_BUNDLE
        .lock()
        .ok()
        .and_then(|guard| guard.clone());
    let Some(bundle_id) = bundle_id else {
        return;
    };

    let ns_bundle = NSString::from_str(&bundle_id);
    let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&ns_bundle);
    if apps.count() == 0 {
        return;
    }

    let app = apps.objectAtIndex(0);
    let _ = app.activateWithOptions(NSApplicationActivationOptions::empty());
}
