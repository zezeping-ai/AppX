use rusqlite::Row;

use super::super::cache::{icon_url_for_bundle, relative_time, thumb_url_for_id};
use super::super::model::{
    ContentBadge, ContentType, ItemSummary, ListItemsQuery, ListItemsResult, PayloadMeta,
};
use super::super::state::ClipboardAssistantState;

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
    let payload_meta_raw: String = row.get("payload_meta")?;
    let payload_meta = PayloadMeta::parse(&payload_meta_raw);
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
        has_rich_format: payload_meta.has_rich_format,
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
    let fts_match = keyword_raw.as_deref().and_then(super::super::fts::build_fts_match);
    let use_fts = fts_match.is_some() && super::super::fts::table_has_fts(&conn);

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
