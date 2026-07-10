//! 前台应用检测：AppX 自身获得焦点时不展开缩写，避免编辑表单时误触发。

use objc2_app_kit::NSWorkspace;

pub fn is_own_app_foreground(own_bundle_id: &str) -> bool {
    if own_bundle_id.is_empty() {
        return false;
    }

    let workspace = NSWorkspace::sharedWorkspace();
    let Some(app) = workspace.frontmostApplication() else {
        return false;
    };
    let Some(bundle) = app.bundleIdentifier() else {
        return false;
    };

    bundle.to_string() == own_bundle_id
}
