//! 剪切助手音效：系统复制 / 浮层拷贝粘贴均在 Rust 侧播放，不依赖 WebView Audio。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use tauri::{AppHandle, Manager};

use super::settings;

const DEFAULT_COPY_MP3: &[u8] =
    include_bytes!("../../../../src/assets/sounds/clipboard/copy.mp3");
const DEFAULT_PASTE_MP3: &[u8] =
    include_bytes!("../../../../src/assets/sounds/clipboard/paste.mp3");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundKind {
    Copy,
    Paste,
}

impl SoundKind {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "copy" => Some(Self::Copy),
            "paste" => Some(Self::Paste),
            _ => None,
        }
    }

    fn default_bytes(self) -> &'static [u8] {
        match self {
            Self::Copy => DEFAULT_COPY_MP3,
            Self::Paste => DEFAULT_PASTE_MP3,
        }
    }

    fn default_file_name(self) -> &'static str {
        match self {
            Self::Copy => "copy.mp3",
            Self::Paste => "paste.mp3",
        }
    }
}

/// 按设置播放；`path_override` 非空则优先（偏好试听未保存路径）。
/// `force` 为真时忽略开关（试听）。
pub fn play(app: &AppHandle, kind: SoundKind, path_override: Option<&str>, force: bool) {
    let settings = settings::read_settings(app).unwrap_or_default();
    if !force {
        let enabled = match kind {
            SoundKind::Copy => settings.copy_sound_enabled,
            SoundKind::Paste => settings.paste_sound_enabled,
        };
        if !enabled {
            return;
        }
    }

    let custom = path_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            let stored = match kind {
                SoundKind::Copy => settings.copy_sound_path,
                SoundKind::Paste => settings.paste_sound_path,
            };
            let trimmed = stored.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

    let path = match custom {
        Some(p) => PathBuf::from(p),
        None => match ensure_default_file(app, kind) {
            Ok(p) => p,
            Err(err) => {
                eprintln!("[clipboard_sound] prepare default failed: {err}");
                return;
            }
        },
    };

    if !path.is_file() {
        eprintln!("[clipboard_sound] missing file: {}", path.display());
        return;
    }
    spawn_player(&path);
}

fn ensure_default_file(app: &AppHandle, kind: SoundKind) -> Result<PathBuf, String> {
    static COPY_PATH: OnceLock<PathBuf> = OnceLock::new();
    static PASTE_PATH: OnceLock<PathBuf> = OnceLock::new();

    let slot = match kind {
        SoundKind::Copy => &COPY_PATH,
        SoundKind::Paste => &PASTE_PATH,
    };
    if let Some(path) = slot.get() {
        return Ok(path.clone());
    }

    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("cache dir: {e}"))?
        .join("clipboard-sounds");
    fs::create_dir_all(&dir).map_err(|e| format!("create sound cache: {e}"))?;
    let path = dir.join(kind.default_file_name());
    if !path.is_file() || fs::metadata(&path).map(|m| m.len()).unwrap_or(0) == 0 {
        fs::write(&path, kind.default_bytes()).map_err(|e| format!("write default sound: {e}"))?;
    }
    let _ = slot.set(path.clone());
    Ok(path)
}

fn spawn_player(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("afplay").arg(path).spawn();
    }

    #[cfg(target_os = "windows")]
    {
        // MediaPlayer 可播 mp3；短音效后台启动即可
        let path_str = path.display().to_string().replace('\'', "''");
        let script = format!(
            "Add-Type -AssemblyName presentationCore; \
             $p = New-Object System.Windows.Media.MediaPlayer; \
             $p.Open([uri]'{path_str}'); \
             $p.Play(); \
             Start-Sleep -Milliseconds 1200"
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .spawn();
        return;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if Command::new("ffplay")
            .args(["-nodisp", "-autoexit", "-loglevel", "quiet"])
            .arg(path)
            .spawn()
            .is_ok()
        {
            return;
        }
        let _ = Command::new("paplay").arg(path).spawn();
    }
}
