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

    pub fn rename_key(&self, old_path: &str, new_path: &str) {
        if old_path == new_path {
            return;
        }
        if let Ok(mut guard) = self.map.lock() {
            if let Some(passphrase) = guard.remove(old_path) {
                guard.insert(new_path.to_string(), passphrase);
            }
        }
    }
}
