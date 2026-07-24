use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::app::app_lock::is_session_locked;
use crate::app::clipboard;

use super::classifier::{classify, maybe_compress_image};
use super::reader::{fingerprint, frontmost_app, read_once};
use super::super::history::ingest_capture;
use super::super::settings;
use super::super::state::ClipboardAssistantState;

static MONITORING: AtomicBool = AtomicBool::new(false);
static TX: Mutex<Option<Sender<()>>> = Mutex::new(None);
static MONITOR_HANDLE: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);

pub fn start_monitoring(app: AppHandle, state: Arc<ClipboardAssistantState>) -> Result<(), String> {
    stop_monitoring();
    let (tx, rx) = mpsc::channel();
    *TX.lock().map_err(|_| "监控通道锁失败".to_string())? = Some(tx);
    MONITORING.store(true, Ordering::Relaxed);

    let handle = thread::spawn(move || {
        // 以当前剪贴板为基线：启动/解锁重启只对齐状态，不视为一次「新复制」
        let mut last_fp = read_once()
            .ok()
            .map(|read| fingerprint(&read))
            .unwrap_or_default();

        while MONITORING.load(Ordering::Relaxed) {
            if rx.try_recv().is_ok() {
                break;
            }
            // suppress：本进程粘贴会话写板中；Transient：OS 瞬时标记（不改 last_fp）
            if is_session_locked(&app)
                || !settings::is_monitoring_enabled()
                || clipboard::is_record_suppressed()
                || clipboard::transient::is_marked_transient()
            {
                thread::sleep(Duration::from_millis(350));
                continue;
            }
            if let Ok(read) = read_once() {
                let fp = fingerprint(&read);
                if fp == last_fp {
                    thread::sleep(Duration::from_millis(350));
                    continue;
                }
                last_fp = fp;
                handle_clipboard_change(&app, &state, read);
            }
            thread::sleep(Duration::from_millis(350));
        }
    });

    *MONITOR_HANDLE
        .lock()
        .map_err(|_| "监控线程锁失败".to_string())? = Some(handle);

    Ok(())
}

fn handle_clipboard_change(
    app: &AppHandle,
    state: &Arc<ClipboardAssistantState>,
    read: super::reader::ClipboardRead,
) {
    let Some(mut payload) = classify(read) else {
        return;
    };
    if let Some(bytes) = payload.image_bytes.take() {
        let dimensions = payload.image_dimensions.take();
        let Ok(bytes) = super::super::thumb::normalize_image_bytes(bytes, dimensions) else {
            return;
        };
        let snap = state
            .settings()
            .unwrap_or_else(|_| super::super::settings::snapshot(&state.app));
        let Some(bytes) = maybe_compress_image(
            bytes,
            snap.max_image_blob_bytes,
            snap.max_image_blob_hard_bytes,
            snap.compress_oversized_images,
        ) else {
            return;
        };
        payload.image_bytes = Some(bytes);
    }
    let (bundle, name) = frontmost_app();
    let _ = ingest_capture(state, payload, bundle, name);
    // 仅真实变化（相对基线）才播复制音效
    super::super::sounds::play(app, super::super::sounds::SoundKind::Copy, None, false);
    let _ = app.emit("appx:clipboard-changed", ());
}

pub fn stop_monitoring() {
    MONITORING.store(false, Ordering::Relaxed);
    if let Ok(mut guard) = TX.lock() {
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }
    }
    if let Ok(mut guard) = MONITOR_HANDLE.lock() {
        if let Some(handle) = guard.take() {
            let _ = handle.join();
        }
    }
}

pub fn restart_monitoring(app: &AppHandle) -> Result<(), String> {
    stop_monitoring();
    if is_session_locked(app) || !settings::is_monitoring_enabled() {
        return Ok(());
    }
    if let Some(state) = app.try_state::<Arc<ClipboardAssistantState>>() {
        let state = state.inner().clone();
        start_monitoring(app.clone(), state)?;
    }
    Ok(())
}
