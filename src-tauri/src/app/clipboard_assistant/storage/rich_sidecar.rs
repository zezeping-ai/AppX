use rusqlite::{params, Connection};

use super::super::db::{read_rich_formats, rich_path, write_rich_formats};
use super::super::model::PayloadMeta;
use super::super::state::ClipboardAssistantState;
use crate::app::clipboard::rich::RichFormats;

pub fn rich_formats_size(formats: &RichFormats) -> usize {
    formats.html.as_ref().map(|value| value.len()).unwrap_or(0)
        + formats.rtf.as_ref().map(|value| value.len()).unwrap_or(0)
}

pub fn persist_rich_formats(
    state: &ClipboardAssistantState,
    conn: &Connection,
    id: i64,
    formats: &RichFormats,
) -> Result<(), String> {
    write_rich_formats(&rich_path(&state.blobs_dir, id), formats)?;
    let meta = PayloadMeta::from_rich(Some(formats));
    conn.execute(
        "UPDATE clipboard_items SET payload_meta = ?1 WHERE id = ?2",
        params![meta.to_json(), id],
    )
    .map_err(|e| format!("更新富文本元数据失败：{e}"))?;
    Ok(())
}

pub fn load_rich_formats(
    state: &ClipboardAssistantState,
    id: i64,
) -> Result<Option<RichFormats>, String> {
    read_rich_formats(&rich_path(&state.blobs_dir, id))
}
