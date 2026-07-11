use std::fs;
use std::path::Path;

use rusqlite::{params, Connection};

use super::db::{blob_path, delete_items_by_ids, thumb_path};
use super::state::ClipboardAssistantState;

pub fn remove_ids(state: &ClipboardAssistantState, ids: &[i64]) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }

    let pinned_flags: Vec<bool> = {
        let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
        ids.iter()
            .map(|id| {
                conn.query_row(
                    "SELECT pinned FROM clipboard_items WHERE id = ?1",
                    params![id],
                    |row| row.get::<_, i64>(0),
                )
                .map(|v| v == 1)
                .unwrap_or(false)
            })
            .collect()
    };

    {
        let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
        delete_items_by_ids(&conn, ids)?;
    }

    for (id, pinned) in ids.iter().zip(pinned_flags.iter()) {
        let _ = fs::remove_file(blob_path(&state.blobs_dir, *id));
        let _ = fs::remove_file(thumb_path(&state.blobs_dir, *id));
        state.dec_counts(*pinned);
    }

    if let Ok(mut cache) = state.cache.write() {
        cache.remove_ids(ids);
    }
    Ok(())
}

pub fn prune_unpinned(state: &ClipboardAssistantState, max: u32) -> Result<(), String> {
    let max = max.max(50) as i64;
    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let unpinned = super::db::count_items(&conn, Some(false))? as i64;
    if unpinned <= max {
        return Ok(());
    }
    let excess = (unpinned - max) as usize;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM clipboard_items WHERE pinned = 0 ORDER BY created_at ASC LIMIT ?1",
        )
        .map_err(|e| format!("查询待清理条目失败：{e}"))?;
    let ids: Vec<i64> = stmt
        .query_map(params![excess as i64], |row| row.get(0))
        .map_err(|e| format!("遍历待清理条目失败：{e}"))?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);
    drop(conn);
    remove_ids(state, &ids)
}

pub fn sweep_orphan_blobs(blobs_dir: &Path, conn: &Connection) -> Result<u32, String> {
    let mut removed = 0u32;
    let entries = fs::read_dir(blobs_dir).map_err(|e| format!("读取 blob 目录失败：{e}"))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let id_str = name.split('.').next().unwrap_or("").split('_').next().unwrap_or("");
        let Ok(id) = id_str.parse::<i64>() else {
            continue;
        };
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM clipboard_items WHERE id = ?1",
                params![id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !exists {
            let _ = fs::remove_file(entry.path());
            removed += 1;
        }
    }
    Ok(removed)
}

pub fn blob_dir_size(blobs_dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(blobs_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}
