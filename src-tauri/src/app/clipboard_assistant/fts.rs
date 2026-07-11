/// FTS5 查询构造与搜索辅助
pub fn build_fts_match(keyword: &str) -> Option<String> {
    let terms: Vec<String> = keyword
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| {
            let escaped = t.replace('"', "");
            format!("\"{escaped}\"*")
        })
        .collect();
    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" AND "))
    }
}

pub fn table_has_fts(conn: &rusqlite::Connection) -> bool {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='clipboard_fts' LIMIT 1",
        [],
        |_| Ok(()),
    )
    .is_ok()
}
