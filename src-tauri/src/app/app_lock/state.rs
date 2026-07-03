use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct AppLockSessionState {
    locked: Mutex<bool>,
}

impl AppLockSessionState {
    pub fn is_locked(&self) -> Result<bool, String> {
        self.locked
            .lock()
            .map(|guard| *guard)
            .map_err(|_| "应用锁状态已损坏".to_string())
    }

    pub fn set_locked(&self, locked: bool) -> Result<(), String> {
        let mut guard = self
            .locked
            .lock()
            .map_err(|_| "应用锁状态已损坏".to_string())?;
        *guard = locked;
        Ok(())
    }
}
