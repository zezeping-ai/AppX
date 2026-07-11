CREATE TABLE IF NOT EXISTS clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,
    preview TEXT NOT NULL,
    source_app_bundle TEXT,
    source_app_name TEXT,
    group_key TEXT NOT NULL DEFAULT 'default',
    pinned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    payload_kind TEXT NOT NULL,
    payload_ref TEXT,
    char_count INTEGER,
    accent_color TEXT NOT NULL DEFAULT '#6b7280',
    tags TEXT NOT NULL DEFAULT '[]',
    payload_meta TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_clipboard_items_list
    ON clipboard_items (pinned DESC, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_source
    ON clipboard_items (source_app_bundle, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_group
    ON clipboard_items (group_key, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_hash
    ON clipboard_items (content_hash, created_at DESC);

CREATE TABLE IF NOT EXISTS clipboard_text (
    item_id INTEGER PRIMARY KEY REFERENCES clipboard_items(id) ON DELETE CASCADE,
    content TEXT NOT NULL
);
