use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use tauri::Manager;

pub(super) fn append_update_log(app: &tauri::AppHandle, stage: &str, message: impl AsRef<str>) {
    let Ok(log_dir) = app.path().app_log_dir() else {
        return;
    };
    if create_dir_all(&log_dir).is_err() {
        return;
    }

    let log_path = log_dir.join("updater.log");
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) else {
        return;
    };

    let at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or_default();

    let _ = writeln!(file, "[{at_ms}] [{stage}] {}", message.as_ref());
}
