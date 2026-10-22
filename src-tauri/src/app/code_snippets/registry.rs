use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::model::{PaletteItem, SnippetEntry};

#[derive(Default)]
pub struct SnippetRegistry {
    inner: Arc<RwLock<RegistrySnapshot>>,
    /// 懒解密缓存：`(snippet_id, app_key)` → 明文；`app_key` 空串表示默认内容
    plaintext: Arc<RwLock<HashMap<(i64, String), String>>>,
}

/// 仅索引：缩写 / id / 快捷键 → 条目元数据（无正文）
#[derive(Default, Clone)]
pub struct RegistrySnapshot {
    pub by_abbreviation: HashMap<String, SnippetEntry>,
    pub by_id: HashMap<i64, SnippetEntry>,
    pub shortcuts: Vec<(String, SnippetEntry)>,
}

impl SnippetRegistry {
    pub fn replace(&self, entries: Vec<SnippetEntry>) {
        let mut by_abbreviation = HashMap::new();
        let mut by_id = HashMap::new();
        let mut shortcuts = Vec::new();

        for entry in entries {
            if !entry.abbreviation.trim().is_empty() {
                let key = entry.abbreviation.trim().to_lowercase();
                by_abbreviation.insert(key, entry.clone());
            }
            by_id.insert(entry.id, entry.clone());
            if let Some(shortcut) = entry
                .shortcut
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            {
                shortcuts.push((shortcut.to_string(), entry));
            }
        }

        let snapshot = RegistrySnapshot {
            by_abbreviation,
            by_id,
            shortcuts,
        };

        if let Ok(mut guard) = self.inner.write() {
            *guard = snapshot;
        }
        // 索引变更后正文可能已改，丢弃懒解密缓存
        self.clear_plaintext_cache();
    }

    pub fn snapshot(&self) -> RegistrySnapshot {
        self.inner
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn cached_plaintext(&self, id: i64, app_key: &str) -> Option<String> {
        self.plaintext
            .read()
            .ok()?
            .get(&(id, app_key.to_string()))
            .cloned()
    }

    pub fn store_plaintext(&self, id: i64, app_key: &str, plain: String) {
        if let Ok(mut guard) = self.plaintext.write() {
            guard.insert((id, app_key.to_string()), plain);
        }
    }

    pub fn clear_plaintext_cache(&self) {
        if let Ok(mut guard) = self.plaintext.write() {
            guard.clear();
        }
    }

    pub fn palette_items(&self) -> Vec<PaletteItem> {
        let mut items: Vec<PaletteItem> = self
            .snapshot()
            .by_id
            .values()
            .map(|entry| PaletteItem {
                id: entry.id,
                name: entry.name.clone(),
                abbreviation: entry.abbreviation.clone(),
                group: entry.group.clone(),
            })
            .collect();
        items.sort_by(|a, b| {
            a.group
                .cmp(&b.group)
                .then_with(|| a.name.cmp(&b.name))
        });
        items
    }
}
