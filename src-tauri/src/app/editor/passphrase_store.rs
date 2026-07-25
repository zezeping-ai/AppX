use std::collections::HashMap;
use std::sync::Mutex;

/// 会话内文件路径 -> 独立加密口令（不落盘，重启后清空）。
#[derive(Debug, Default)]
pub struct FilePassphraseStore {
    map: Mutex<HashMap<String, String>>,
}

impl FilePassphraseStore {
    pub fn get(&self, path: &str) -> Option<String> {
        self.map
            .lock()
            .ok()
            .and_then(|guard| guard.get(path).cloned())
    }

    pub fn set(&self, path: &str, passphrase: String) {
        if let Ok(mut guard) = self.map.lock() {
            guard.insert(path.to_string(), passphrase);
        }
    }

    pub fn remove(&self, path: &str) {
        if let Ok(mut guard) = self.map.lock() {
            guard.remove(path);
        }
    }

    /// 移动/重命名后，按路径前缀批量重映射口令键（含自身）。
    pub fn rename_key_prefix(&self, old_prefix: &str, new_prefix: &str) {
        if old_prefix == new_prefix {
            return;
        }
        if let Ok(mut guard) = self.map.lock() {
            let keys: Vec<String> = guard
                .keys()
                .filter(|key| {
                    *key == old_prefix
                        || key.starts_with(&format!("{old_prefix}/"))
                        || key.starts_with(&format!("{old_prefix}\\"))
                })
                .cloned()
                .collect();
            for old_key in keys {
                if let Some(passphrase) = guard.remove(&old_key) {
                    let suffix = &old_key[old_prefix.len()..];
                    guard.insert(format!("{new_prefix}{suffix}"), passphrase);
                }
            }
        }
    }
}
