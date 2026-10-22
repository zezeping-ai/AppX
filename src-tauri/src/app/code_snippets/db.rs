//! 触发时按 id 读主库 code_snippets（与前端 ActiveRecord 同库；schema 由 database migrations 管理）。

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};

use super::model::{AppVariantRow, SnippetStoredPayload};

/// 复用连接，避免每次触发 open + WAL pragma
#[derive(Default)]
pub struct SnippetDb {
    conn: Mutex<Option<Connection>>,
}

impl SnippetDb {
    pub fn load_payload(&self, app: &AppHandle, id: i64) -> Result<SnippetStoredPayload, String> {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| "代码段数据库锁异常".to_string())?;
        if guard.is_none() {
            let path = crate::paths::database_file_path(app)?;
            *guard = Some(open_db(&path)?);
        }
        let conn = guard.as_ref().expect("conn just initialized");
        query_payload(conn, id)
    }

    pub fn reset(&self) {
        if let Ok(mut guard) = self.conn.lock() {
            *guard = None;
        }
    }
}

pub fn load_snippet_payload(app: &AppHandle, id: i64) -> Result<SnippetStoredPayload, String> {
    if let Some(db) = app.try_state::<SnippetDb>() {
        return db.load_payload(app, id);
    }
    let path = crate::paths::database_file_path(app)?;
    let conn = open_db(&path)?;
    query_payload(&conn, id)
}

fn query_payload(conn: &Connection, id: i64) -> Result<SnippetStoredPayload, String> {
    let (content, variants_json): (String, String) = conn
        .query_row(
            "SELECT content, COALESCE(app_variants, '[]') FROM code_snippets WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => format!("代码段 #{id} 不存在"),
            other => format!("读取代码段 #{id} 失败：{other}"),
        })?;

    let variants: Vec<AppVariantRow> = serde_json::from_str(&variants_json).unwrap_or_default();
    Ok(SnippetStoredPayload { content, variants })
}

fn open_db(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败：{e}"))?;
    }
    let conn = Connection::open(path).map_err(|e| format!("打开数据库失败：{e}"))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("设置 WAL 失败：{e}"))?;
    Ok(conn)
}
