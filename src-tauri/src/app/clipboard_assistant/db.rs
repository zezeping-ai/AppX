use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

/// 与 migrations 同源；Rust 启动时可能早于前端 Database.load()，此处幂等补跑。
const CLIPBOARD_ASSISTANT_MIGRATION_SQL: &str =
    include_str!("../../../migrations/0003_create_clipboard_assistant.sql");
const CLIPBOARD_FTS_MIGRATION_SQL: &str =
    include_str!("../../../migrations/0004_create_clipboard_fts.sql");

pub fn open_db(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败：{e}"))?;
    }
    let conn = Connection::open(path).map_err(|e| format!("打开数据库失败：{e}"))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("设置 WAL 失败：{e}"))?;
    apply_clipboard_schema(&conn)?;
    Ok(conn)
}

/// 使用主库 `appx.db`（与 code_snippets 等同库，迁移在 `database::migrations` 统一登记）。
pub fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    crate::paths::database_file_path(app)
}

pub fn blobs_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    crate::app::settings::storage::app_data_dir(app).map(|d| d.join("clipboard_blobs"))
}

pub fn icons_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    crate::app::settings::storage::app_data_dir(app).map(|d| d.join("clipboard_app_icons"))
}

fn apply_clipboard_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(CLIPBOARD_ASSISTANT_MIGRATION_SQL)
        .map_err(|e| format!("应用剪切助手 schema 失败：{e}"))?;
    conn.execute_batch(CLIPBOARD_FTS_MIGRATION_SQL)
        .map_err(|e| format!("应用剪切助手 FTS 失败：{e}"))
}

pub fn blob_path(blobs: &Path, id: i64) -> PathBuf {
    blobs.join(format!("{id}.bin"))
}

pub fn thumb_path(blobs: &Path, id: i64) -> PathBuf {
    blobs.join(format!("{id}_thumb.webp"))
}

pub fn count_items(conn: &Connection, pinned_only: Option<bool>) -> Result<u64, String> {
    let sql = match pinned_only {
        Some(true) => "SELECT COUNT(*) FROM clipboard_items WHERE pinned = 1",
        Some(false) => "SELECT COUNT(*) FROM clipboard_items WHERE pinned = 0",
        None => "SELECT COUNT(*) FROM clipboard_items",
    };
    conn.query_row(sql, [], |row| row.get(0))
        .map_err(|e| format!("统计条目失败：{e}"))
}

pub fn delete_items_by_ids(conn: &Connection, ids: &[i64]) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("DELETE FROM clipboard_items WHERE id IN ({placeholders})");
    let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
    conn.execute(&sql, params.as_slice())
        .map_err(|e| format!("删除条目失败：{e}"))?;
    Ok(())
}

pub fn get_inline_text(conn: &Connection, id: i64) -> Result<Option<String>, String> {
    match conn.query_row(
        "SELECT content FROM clipboard_text WHERE item_id = ?1",
        params![id],
        |row| row.get::<_, String>(0),
    ) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("读取文本失败：{e}")),
    }
}

/// 按最新规则重建 inline 文本 preview（兼容旧版仅首行预览）。
pub fn refresh_text_previews(conn: &Connection) -> Result<u32, String> {
    use super::enricher::make_preview;

    let mut stmt = conn
        .prepare(
            "SELECT ci.id, ct.content FROM clipboard_items ci
             INNER JOIN clipboard_text ct ON ct.item_id = ci.id
             WHERE ci.payload_kind = 'inline'",
        )
        .map_err(|e| format!("准备 preview 回填失败：{e}"))?;

    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| format!("读取 preview 回填数据失败：{e}"))?;

    let mut updated = 0u32;
    for row in rows {
        let (id, content) = row.map_err(|e| format!("解析 preview 回填行失败：{e}"))?;
        let preview = make_preview(&content);
        let changed = conn
            .execute(
                "UPDATE clipboard_items SET preview = ?1 WHERE id = ?2 AND preview != ?1",
                params![preview, id],
            )
            .map_err(|e| format!("更新 preview 失败：{e}"))?;
        if changed > 0 {
            updated += 1;
        }
    }
    Ok(updated)
}
