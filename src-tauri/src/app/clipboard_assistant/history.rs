use std::fs;

use chrono::Utc;
use rusqlite::{params, Row};

use super::cache::{icon_url_for_bundle, relative_time, thumb_url_for_id};
use super::enricher::{enrich_file, enrich_image, enrich_text, hash_bytes};
use super::model::{
    CapturedPayload, ContentBadge, ContentType, GetContentResult, ItemSummary, ListItemsQuery,
    ListItemsResult, PayloadKind,
};
use super::state::ClipboardAssistantState;

pub fn row_to_summary(row: &Row<'_>) -> Result<ItemSummary, rusqlite::Error> {
    let content_type_raw: String = row.get("content_type")?;
    let content_type = ContentType::parse(&content_type_raw).unwrap_or(ContentType::Text);
    let bundle: Option<String> = row.get("source_app_bundle")?;
    let id: i64 = row.get("id")?;
    let tags_raw: String = row.get("tags")?;
    let badges: Vec<ContentBadge> = serde_json::from_str(&tags_raw).unwrap_or_default();
    let payload_kind_raw: String = row.get("payload_kind")?;
    let thumb_url = if payload_kind_raw == "blob" || payload_kind_raw == "file_ref" {
        Some(thumb_url_for_id(id))
    } else {
        None
    };
    Ok(ItemSummary {
        id,
        content_type,
        preview: row.get("preview")?,
        source_app_bundle: bundle.clone(),
        source_app_name: row.get("source_app_name")?,
        source_app_icon_url: bundle.as_ref().map(|b| icon_url_for_bundle(b)),
        group_key: row.get("group_key")?,
        pinned: row.get::<_, i64>("pinned")? == 1,
        created_at: row.get("created_at")?,
        accent_color: row.get("accent_color")?,
        char_count: row.get("char_count")?,
        tags: vec![],
        badges,
        thumb_url,
        relative_time: relative_time(&row.get::<_, String>("created_at")?),
    })
}

pub fn list_items(
    state: &ClipboardAssistantState,
    query: ListItemsQuery,
) -> Result<ListItemsResult, String> {
    let limit = query.limit.unwrap_or(80).min(200) as i64;
    let offset = query.offset.unwrap_or(0) as i64;
    let has_filter = query.keyword.as_ref().is_some_and(|k| !k.trim().is_empty())
        || query.content_type.is_some()
        || query.group_key.as_ref().is_some_and(|s| !s.is_empty())
        || query.source_app_bundle.as_ref().is_some_and(|s| !s.is_empty())
        || query.pinned_only.is_some()
        || offset > 0;

    if query.prefer_cache.unwrap_or(false) && !has_filter {
        let cache = state.cache.read().map_err(|_| "缓存锁失败".to_string())?;
        let items = cache.list(limit as usize);
        let total = state.total_count.load(std::sync::atomic::Ordering::Relaxed);
        return Ok(ListItemsResult { items, total });
    }

    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let keyword_raw = query.keyword.filter(|k| !k.trim().is_empty());
    let fts_match = keyword_raw.as_deref().and_then(super::fts::build_fts_match);
    let use_fts = fts_match.is_some() && super::fts::table_has_fts(&conn);

    let mut sql = if use_fts {
        String::from(
            "SELECT ci.* FROM clipboard_items ci \
             INNER JOIN (SELECT DISTINCT item_id FROM clipboard_fts WHERE content MATCH ?1) fts \
             ON fts.item_id = ci.id WHERE 1=1",
        )
    } else {
        String::from("SELECT * FROM clipboard_items WHERE 1=1")
    };
    let mut bind_values: Vec<rusqlite::types::Value> = vec![];

    if let Some(ref match_query) = fts_match {
        bind_values.push(rusqlite::types::Value::Text(match_query.clone()));
    }

    let col = if use_fts { "ci." } else { "" };

    if let Some(ct) = query.content_type {
        sql.push_str(&format!(" AND {col}content_type = ?"));
        bind_values.push(rusqlite::types::Value::Text(ct.as_str().to_string()));
    }
    if let Some(gk) = query.group_key.filter(|s| !s.is_empty()) {
        sql.push_str(&format!(" AND {col}group_key = ?"));
        bind_values.push(rusqlite::types::Value::Text(gk));
    }
    if let Some(bundle) = query.source_app_bundle.filter(|s| !s.is_empty()) {
        sql.push_str(&format!(" AND {col}source_app_bundle = ?"));
        bind_values.push(rusqlite::types::Value::Text(bundle));
    }
    if query.pinned_only == Some(true) {
        sql.push_str(&format!(" AND {col}pinned = 1"));
    }
    if !use_fts {
        if let Some(keyword) = keyword_raw {
            sql.push_str(&format!(" AND LOWER({col}preview) LIKE ?"));
            bind_values.push(rusqlite::types::Value::Text(format!(
                "%{}%",
                keyword.trim().to_lowercase()
            )));
        }
    }

    let total = if has_filter {
        let count_sql = sql
            .replace("SELECT ci.*", "SELECT COUNT(*)")
            .replace("SELECT *", "SELECT COUNT(*)");
        let count_params: Vec<&dyn rusqlite::ToSql> =
            bind_values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
        conn.query_row(count_sql.as_str(), count_params.as_slice(), |row| row.get::<_, i64>(0))
            .map_err(|e| format!("计数失败：{e}"))? as u64
    } else {
        state.total_count.load(std::sync::atomic::Ordering::Relaxed)
    };

    if use_fts {
        sql.push_str(" ORDER BY ci.pinned DESC, ci.created_at DESC LIMIT ? OFFSET ?");
    } else {
        sql.push_str(" ORDER BY pinned DESC, created_at DESC LIMIT ? OFFSET ?");
    }
    bind_values.push(rusqlite::types::Value::Integer(limit));
    bind_values.push(rusqlite::types::Value::Integer(offset));

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("查询失败：{e}"))?;
    let params: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(params.as_slice(), row_to_summary)
        .map_err(|e| format!("查询失败：{e}"))?;

    let mut items = vec![];
    for row in rows {
        items.push(row.map_err(|e| format!("解析行失败：{e}"))?);
    }

    Ok(ListItemsResult { items, total })
}

pub fn ingest_capture(
    state: &ClipboardAssistantState,
    payload: CapturedPayload,
    source_bundle: Option<String>,
    source_name: Option<String>,
) -> Result<(), String> {
    if let Some(bundle) = &source_bundle {
        let excluded = state.settings()?.excluded_apps.iter().any(|b| b == bundle);
        if excluded {
            return Ok(());
        }
    }

    let settings = state.settings()?;

    let (kind, meta, inline_text, file_paths, image_bytes) = match payload {
        CapturedPayload {
            kind: PayloadKind::FileRef,
            file_paths: Some(paths),
            ..
        } if !paths.is_empty() => {
            let meta = enrich_file(&paths, source_bundle.as_deref());
            (PayloadKind::FileRef, meta, None, Some(paths), None)
        }
        CapturedPayload {
            kind: PayloadKind::Blob,
            image_bytes: Some(bytes),
            ..
        } => {
            let mut meta = enrich_image(source_bundle.as_deref());
            meta.content_hash = hash_bytes(&bytes);
            (PayloadKind::Blob, meta, None, None, Some(bytes))
        }
        CapturedPayload {
            text: Some(text),
            ..
        } => {
            let bytes = text.as_bytes();
            if bytes.len() as u32 > settings.max_text_bytes {
                return Ok(());
            }
            let meta = enrich_text(&text, source_bundle.as_deref());
            if bytes.len() > settings.text_inline_threshold as usize {
                (PayloadKind::Blob, meta, None, None, Some(bytes.to_vec()))
            } else {
                (PayloadKind::Inline, meta, Some(text), None, None)
            }
        }
        _ => return Ok(()),
    };

    if settings.dedupe_mode == "consecutive" {
        if let Ok(cache) = state.cache.read() {
            if cache.last_hash().as_deref() == Some(&meta.content_hash) {
                return Ok(());
            }
        }
    }

    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    if let Ok(existing_id) = conn.query_row(
        "SELECT id FROM clipboard_items WHERE content_hash = ?1 ORDER BY created_at DESC LIMIT 1",
        params![meta.content_hash],
        |row| row.get::<_, i64>(0),
    ) {
        drop(conn);
        return touch_item(state, existing_id);
    }

    let created_at = Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&meta.badges).unwrap_or_else(|_| "[]".into());
    let payload_ref = file_paths
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap_or_default());

    conn.execute(
        "INSERT INTO clipboard_items (
            content_type, preview, source_app_bundle, source_app_name, group_key, pinned,
            created_at, content_hash, payload_kind, payload_ref, char_count, accent_color, tags, payload_meta
        ) VALUES (?1,?2,?3,?4,?5,0,?6,?7,?8,?9,?10,?11,?12,'{}')",
        params![
            meta.content_type.as_str(),
            meta.preview,
            source_bundle,
            source_name,
            meta.group_key,
            created_at,
            meta.content_hash,
            kind.as_str(),
            payload_ref,
            meta.char_count,
            meta.accent_color,
            tags_json,
        ],
    )
    .map_err(|e| format!("写入条目失败：{e}"))?;
    let id = conn.last_insert_rowid();

    if let Some(text) = inline_text {
        conn.execute(
            "INSERT INTO clipboard_text (item_id, content) VALUES (?1, ?2)",
            params![id, text],
        )
        .map_err(|e| format!("写入文本失败：{e}"))?;
    }

    if let Some(bytes) = image_bytes {
        let path = super::db::blob_path(&state.blobs_dir, id);
        fs::write(&path, &bytes).map_err(|e| format!("写入 blob 失败：{e}"))?;
        let _ = super::thumb::write_thumb_from_bytes(&state.blobs_dir, id, &bytes);
    } else if let Some(paths) = &file_paths {
        if let Some(first) = paths.first() {
            let _ = super::thumb::write_thumb_from_path(&state.blobs_dir, id, first);
        }
    }

    let item = conn
        .query_row("SELECT * FROM clipboard_items WHERE id = ?1", params![id], row_to_summary)
        .map_err(|e| format!("读取新条目失败：{e}"))?;

    drop(conn);

    state.inc_counts(false);
    if let Ok(mut cache) = state.cache.write() {
        cache.set_last_hash(Some(meta.content_hash));
        cache.push_front(item);
    }

    super::cleanup::prune_unpinned(state, settings.max_history_items)?;

    if let Some(bundle) = source_bundle.filter(|b| !b.is_empty()) {
        let app = state.app.clone();
        std::thread::spawn(move || {
            let _ = super::app_icon::ensure_cached(&app, &bundle);
        });
    }
    Ok(())
}

/// 将已有条目置顶（更新 created_at，保留原来源应用）。
pub fn touch_item(state: &ClipboardAssistantState, id: i64) -> Result<(), String> {
    let created_at = Utc::now().to_rfc3339();
    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let mut item = conn
        .query_row("SELECT * FROM clipboard_items WHERE id = ?1", params![id], row_to_summary)
        .map_err(|_| format!("条目 #{id} 不存在"))?;
    conn.execute(
        "UPDATE clipboard_items SET created_at = ?1 WHERE id = ?2",
        params![created_at, id],
    )
    .map_err(|e| format!("更新条目时间失败：{e}"))?;
    drop(conn);

    item.created_at = created_at.clone();
    item.relative_time = relative_time(&created_at);
    if let Ok(mut cache) = state.cache.write() {
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

pub type ApplyPayloadData = (PayloadKind, Option<String>, Option<Vec<String>>, Option<Vec<u8>>);

pub fn load_payload_for_apply(
    state: &ClipboardAssistantState,
    id: i64,
) -> Result<ApplyPayloadData, String> {
    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let kind_raw: String = conn
        .query_row(
            "SELECT payload_kind FROM clipboard_items WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|_| format!("条目 #{id} 不存在"))?;
    let kind = PayloadKind::parse(&kind_raw).ok_or_else(|| "未知 payload 类型".to_string())?;
    match kind {
        PayloadKind::Inline => {
            let text = super::db::get_inline_text(&conn, id)?.ok_or_else(|| "文本不存在".to_string())?;
            Ok((kind, Some(text), None, None))
        }
        PayloadKind::Blob => {
            let path = super::db::blob_path(&state.blobs_dir, id);
            drop(conn);
            let bytes = fs::read(&path).map_err(|e| format!("读取 blob 失败：{e}"))?;
            Ok((kind, None, None, Some(bytes)))
        }
        PayloadKind::FileRef => {
            let raw: String = conn
                .query_row(
                    "SELECT payload_ref FROM clipboard_items WHERE id = ?1",
                    params![id],
                    |r| r.get(0),
                )
                .map_err(|e| format!("读取路径失败：{e}"))?;
            let paths: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
            Ok((kind, None, Some(paths), None))
        }
    }
}

pub fn get_content(state: &ClipboardAssistantState, id: i64) -> Result<GetContentResult, String> {
    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let content_type_raw: String = conn
        .query_row(
            "SELECT content_type FROM clipboard_items WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|_| format!("条目 #{id} 不存在"))?;
    drop(conn);
    let content_type = ContentType::parse(&content_type_raw).unwrap_or(ContentType::Text);
    let (kind, text, paths, blob) = load_payload_for_apply(state, id)?;
    Ok(GetContentResult {
        content_type,
        payload_kind: kind,
        text: text.or_else(|| blob.as_ref().and_then(|b| String::from_utf8(b.clone()).ok())),
        file_paths: paths,
        has_blob: blob.is_some(),
    })
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
        let collected: Vec<i64> = rows.filter_map(|r| r.ok()).collect();
        collected
    };
    super::cleanup::remove_ids(state, &ids)?;
    Ok(())
}
