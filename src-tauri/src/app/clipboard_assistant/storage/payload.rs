use std::fs;

use rusqlite::params;

use super::rich_sidecar::load_rich_formats;
use super::super::model::{ContentType, GetContentResult, PayloadKind};
use super::super::state::ClipboardAssistantState;
use crate::app::clipboard::rich::RichFormats;

pub type ApplyPayloadData = (
    PayloadKind,
    Option<String>,
    Option<Vec<String>>,
    Option<Vec<u8>>,
    Option<RichFormats>,
);

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
    let rich = load_rich_formats(state, id)?;
    match kind {
        PayloadKind::Inline => {
            let text = super::super::db::get_inline_text(&conn, id)?.ok_or_else(|| "文本不存在".to_string())?;
            Ok((kind, Some(text), None, None, rich))
        }
        PayloadKind::Blob => {
            let path = super::super::db::blob_path(&state.blobs_dir, id);
            drop(conn);
            let bytes = fs::read(&path).map_err(|e| format!("读取 blob 失败：{e}"))?;
            Ok((kind, None, None, Some(bytes), rich))
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
            Ok((kind, None, Some(paths), None, rich))
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
    let (kind, text, paths, blob, _rich) = load_payload_for_apply(state, id)?;
    Ok(GetContentResult {
        content_type,
        payload_kind: kind,
        text: text.or_else(|| blob.as_ref().and_then(|b| String::from_utf8(b.clone()).ok())),
        file_paths: paths,
        has_blob: blob.is_some(),
    })
}
