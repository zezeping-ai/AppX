use chrono::Utc;
use rusqlite::params;

use super::list::row_to_summary;
use super::rich_sidecar::persist_rich_formats;
use super::super::cache::relative_time;
use super::super::state::ClipboardAssistantState;
use crate::app::clipboard::rich::RichFormats;

/// 将已有条目置顶；若提供富文本则同步更新 sidecar。
pub fn touch_item(
    state: &ClipboardAssistantState,
    id: i64,
    rich_formats: Option<&RichFormats>,
) -> Result<(), String> {
    let created_at = Utc::now().to_rfc3339();
    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let mut item = conn
        .query_row("SELECT * FROM clipboard_items WHERE id = ?1", params![id], row_to_summary)
        .map_err(|_| format!("条目 #{id} 不存在"))?;
    let content_hash: String = conn
        .query_row(
            "SELECT content_hash FROM clipboard_items WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("读取条目 hash 失败：{e}"))?;

    if let Some(formats) = rich_formats.filter(|value| value.has_content()) {
        persist_rich_formats(state, &conn, id, formats)?;
        item.has_rich_format = true;
    }

    conn.execute(
        "UPDATE clipboard_items SET created_at = ?1 WHERE id = ?2",
        params![created_at, id],
    )
    .map_err(|e| format!("更新条目时间失败：{e}"))?;
    drop(conn);

    item.created_at = created_at.clone();
    item.relative_time = relative_time(&created_at);
    if let Ok(mut cache) = state.cache.write() {
        cache.set_last_hash(Some(content_hash));
        cache.push_front(item);
    }
    Ok(())
}

pub fn toggle_pin(state: &ClipboardAssistantState, id: i64) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let pinned: i64 = conn
        .query_row("SELECT pinned FROM clipboard_items WHERE id = ?1", params![id], |r| r.get(0))
        .map_err(|_| format!("条目 #{id} 不存在"))?;
    let next = if pinned == 1 { 0 } else { 1 };
    conn.execute(
        "UPDATE clipboard_items SET pinned = ?1 WHERE id = ?2",
        params![next, id],
    )
    .map_err(|e| format!("更新固定状态失败：{e}"))?;
    drop(conn);
    if next == 1 {
        state.pinned_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        state.unpinned_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    } else {
        state.pinned_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        state.unpinned_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    if let Ok(mut cache) = state.cache.write() {
        cache.update_pin(id, next == 1);
    }
    Ok(next == 1)
}

pub fn warm_cache(state: &ClipboardAssistantState) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let mut stmt = conn
        .prepare("SELECT * FROM clipboard_items ORDER BY pinned DESC, created_at DESC LIMIT 80")
        .map_err(|e| format!("预热缓存失败：{e}"))?;
    let rows = stmt
        .query_map([], row_to_summary)
        .map_err(|e| format!("预热缓存失败：{e}"))?;
    let mut items = vec![];
    for row in rows {
        items.push(row.map_err(|e| format!("预热缓存失败：{e}"))?);
    }
    drop(stmt);
    drop(conn);
    if let Ok(mut cache) = state.cache.write() {
        cache.warm(items.into_iter().rev());
    }
    Ok(())
}

pub fn clear_unpinned(state: &ClipboardAssistantState) -> Result<(), String> {
    let ids: Vec<i64> = {
        let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
        let mut stmt = conn
            .prepare("SELECT id FROM clipboard_items WHERE pinned = 0")
            .map_err(|e| format!("查询失败：{e}"))?;
        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("查询失败：{e}"))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    super::super::cleanup::remove_ids(state, &ids)?;
    Ok(())
}
