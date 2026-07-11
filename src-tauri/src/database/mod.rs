use std::fs::{create_dir_all, OpenOptions};
use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_sql::{Builder, Migration, MigrationKind};

use crate::paths;

pub fn app_db_path() -> &'static str {
    paths::sqlite_connection_string()
}

pub fn db_full_path(app: &AppHandle) -> Result<PathBuf, String> {
    paths::database_file_path(app)
}

fn ensure_db_file(path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .map_err(|err| format!("无法创建数据库目录 {}：{err}", parent.display()))?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map(|_| ())
        .map_err(|err| format!("无法初始化数据库文件 {}：{err}", path.display()))
}

pub fn log_db_full_path(app: &AppHandle) {
    match db_full_path(app) {
        Ok(full_path) => {
            if let Err(err) = ensure_db_file(&full_path) {
                eprintln!("[database] failed to ensure sqlite file: {err}");
            } else {
                log::info!("[database] sqlite ensured: {}", full_path.display());
            }
        }
        Err(err) => eprintln!("[database] failed to resolve database path: {err}"),
    }
}

pub fn plugin() -> Builder {
    Builder::default().add_migrations(app_db_path(), migrations())
}

fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_kv_records",
            sql: include_str!("../../migrations/0001_create_kv_records.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_code_snippets",
            sql: include_str!("../../migrations/0002_create_code_snippets.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_clipboard_assistant",
            sql: include_str!("../../migrations/0003_create_clipboard_assistant.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "create_clipboard_fts",
            sql: include_str!("../../migrations/0004_create_clipboard_fts.sql"),
            kind: MigrationKind::Up,
        },
    ]
}
