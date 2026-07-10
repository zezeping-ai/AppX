use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::model::{PaletteItem, SnippetEntry};

#[derive(Default)]
pub struct SnippetRegistry {
    inner: Arc<RwLock<RegistrySnapshot>>,
}

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
    }

    pub fn snapshot(&self) -> RegistrySnapshot {
        self.inner
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
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
            })
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }
}
