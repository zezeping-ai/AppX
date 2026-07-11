use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};

use super::model::ItemSummary;

const CACHE_LIMIT: usize = 80;

#[derive(Default)]
pub struct HotCache {
    recent: VecDeque<ItemSummary>,
    last_hash: Option<String>,
    revision: AtomicU64,
}

impl HotCache {
    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn bump(&self) {
        self.revision.fetch_add(1, Ordering::Relaxed);
    }

    pub fn last_hash(&self) -> Option<String> {
        self.last_hash.clone()
    }

    pub fn set_last_hash(&mut self, hash: Option<String>) {
        self.last_hash = hash;
    }

    pub fn push_front(&mut self, item: ItemSummary) {
        self.recent.retain(|i| i.id != item.id);
        self.recent.push_front(item);
        while self.recent.len() > CACHE_LIMIT {
            self.recent.pop_back();
        }
        self.bump();
    }

    pub fn remove_ids(&mut self, ids: &[i64]) {
        if ids.is_empty() {
            return;
        }
        self.recent.retain(|i| !ids.contains(&i.id));
        self.bump();
    }

    pub fn update_pin(&mut self, id: i64, pinned: bool) {
        if let Some(item) = self.recent.iter_mut().find(|i| i.id == id) {
            item.pinned = pinned;
            self.bump();
        }
    }

    pub fn list(&self, limit: usize) -> Vec<ItemSummary> {
        self.recent.iter().take(limit).cloned().collect()
    }

    pub fn warm(&mut self, items: impl IntoIterator<Item = ItemSummary>) {
        self.recent.clear();
        for item in items {
            self.recent.push_front(item);
        }
        while self.recent.len() > CACHE_LIMIT {
            self.recent.pop_back();
        }
        self.bump();
    }
}

pub type SharedHotCache = Arc<RwLock<HotCache>>;

pub fn relative_time(iso: &str) -> String {
    let Ok(parsed) = DateTime::parse_from_rfc3339(iso) else {
        return iso.to_string();
    };
    let secs = (Utc::now() - parsed.with_timezone(&Utc)).num_seconds();
    if secs < 60 {
        return "刚刚".to_string();
    }
    if secs < 3600 {
        return format!("{} 分钟前", secs / 60);
    }
    if secs < 86_400 {
        return format!("{} 小时前", secs / 3600);
    }
    format!("{} 天前", secs / 86_400)
}

pub fn icon_url_for_bundle(bundle: &str) -> String {
    super::app_icon::icon_url_for_bundle(bundle)
}

pub fn thumb_url_for_id(id: i64) -> String {
    super::app_icon::thumb_url_for_id(id)
}
