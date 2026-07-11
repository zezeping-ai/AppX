//! 会话锁/解锁事件协调：各功能域注册回调，app_lock 不直接依赖具体功能模块

use std::sync::Mutex;

use tauri::AppHandle;

type SessionCallback = fn(&AppHandle) -> Result<(), String>;

struct Registry {
    on_locked: Vec<SessionCallback>,
    on_unlocked: Vec<SessionCallback>,
}

static REGISTRY: Mutex<Registry> = Mutex::new(Registry {
    on_locked: Vec::new(),
    on_unlocked: Vec::new(),
});

pub fn register_on_locked(callback: SessionCallback) {
    REGISTRY.lock().unwrap().on_locked.push(callback);
}

pub fn register_on_unlocked(callback: SessionCallback) {
    REGISTRY.lock().unwrap().on_unlocked.push(callback);
}

pub fn on_session_locked(app: &AppHandle) -> Result<(), String> {
    for callback in &REGISTRY.lock().unwrap().on_locked {
        callback(app)?;
    }
    Ok(())
}

pub fn on_session_unlocked(app: &AppHandle) -> Result<(), String> {
    for callback in &REGISTRY.lock().unwrap().on_unlocked {
        callback(app)?;
    }
    Ok(())
}
