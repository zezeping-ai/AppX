use std::fs;

use chrono::Utc;
use rusqlite::params;

use super::list::row_to_summary;
use super::mutate::touch_item;
use super::rich_sidecar::{persist_rich_formats, rich_formats_size};
use super::super::enricher::{enrich_file, enrich_image, enrich_text, hash_bytes};
use super::super::model::{CapturedPayload, PayloadKind, PayloadMeta};
use super::super::state::ClipboardAssistantState;

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

    let rich_formats = payload
        .rich_formats
        .clone()
        .filter(|formats| rich_formats_size(formats) as u32 <= settings.max_text_bytes);

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
            let meta = enrich_text(&text, source_bundle.as_deref(), rich_formats.as_ref());
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
        return touch_item(state, existing_id, rich_formats.as_ref());
    }

    let created_at = Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&meta.badges).unwrap_or_else(|_| "[]".into());
    let payload_ref = file_paths
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap_or_default());
    let payload_meta_json = PayloadMeta::from_rich(rich_formats.as_ref()).to_json();

    conn.execute(
        "INSERT INTO clipboard_items (
            content_type, preview, source_app_bundle, source_app_name, group_key, pinned,
            created_at, content_hash, payload_kind, payload_ref, char_count, accent_color, tags, payload_meta
        ) VALUES (?1,?2,?3,?4,?5,0,?6,?7,?8,?9,?10,?11,?12,?13)",
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
            payload_meta_json,
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
        let path = super::super::db::blob_path(&state.blobs_dir, id);
        fs::write(&path, &bytes).map_err(|e| format!("写入 blob 失败：{e}"))?;
        let _ = super::super::thumb::write_thumb_from_bytes(&state.blobs_dir, id, &bytes);
    } else if let Some(paths) = &file_paths {
        if let Some(first) = paths.first() {
            let _ = super::super::thumb::write_thumb_from_path(&state.blobs_dir, id, first);
        }
    }

    if let Some(formats) = rich_formats.filter(|value| value.has_content()) {
        persist_rich_formats(state, &conn, id, &formats)?;
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

    super::super::cleanup::prune_unpinned(state, settings.max_history_items)?;

    if let Some(bundle) = source_bundle.filter(|b| !b.is_empty()) {
        let app = state.app.clone();
        std::thread::spawn(move || {
            let _ = super::super::app_icon::ensure_cached(&app, &bundle);
        });
    }
    Ok(())
}
