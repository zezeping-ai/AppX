use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use rusqlite::Connection;
use tauri::AppHandle;

use super::cache::{HotCache, SharedHotCache};
use super::settings::SettingsSnapshot;

pub struct ClipboardAssistantState {
    pub app: AppHandle,
    pub db: Mutex<Connection>,
    pub cache: SharedHotCache,
    pub settings: RwLock<SettingsSnapshot>,
    pub blobs_dir: PathBuf,
    pub total_count: AtomicU64,
    pub unpinned_count: AtomicU64,
    pub pinned_count: AtomicU64,
}

impl ClipboardAssistantState {
    pub fn new(app: AppHandle, conn: Connection, blobs_dir: PathBuf, settings: SettingsSnapshot) -> Self {
        let total = super::db::count_items(&conn, None).unwrap_or(0);
        let pinned = super::db::count_items(&conn, Some(true)).unwrap_or(0);
        let unpinned = total.saturating_sub(pinned);
        Self {
            app,
            db: Mutex::new(conn),
            cache: Arc::new(RwLock::new(HotCache::default())),
            settings: RwLock::new(settings),
            blobs_dir,
            total_count: AtomicU64::new(total),
            unpinned_count: AtomicU64::new(unpinned),
            pinned_count: AtomicU64::new(pinned),
        }
    }

    pub fn reload_settings(&self) -> Result<(), String> {
        let snap = super::settings::snapshot(&self.app);
        super::settings::apply_runtime_flags(&snap);
        *self.settings.write().map_err(|_| "设置锁失败".to_string())? = snap;
        Ok(())
    }

    pub fn settings(&self) -> Result<SettingsSnapshot, String> {
        Ok(self
            .settings
            .read()
            .map_err(|_| "设置锁失败".to_string())?
            .clone())
    }

    pub fn inc_counts(&self, pinned: bool) {
        self.total_count.fetch_add(1, Ordering::Relaxed);
        if pinned {
            self.pinned_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.unpinned_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn dec_counts(&self, pinned: bool) {
        self.total_count.fetch_sub(1, Ordering::Relaxed);
        if pinned {
            self.pinned_count.fetch_sub(1, Ordering::Relaxed);
        } else {
            self.unpinned_count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
