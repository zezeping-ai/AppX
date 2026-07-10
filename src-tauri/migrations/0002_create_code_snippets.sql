CREATE TABLE IF NOT EXISTS code_snippets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    abbreviation TEXT NOT NULL,
    shortcut TEXT,
    content TEXT NOT NULL,
    meta TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_code_snippets_abbreviation ON code_snippets(abbreviation);
CREATE UNIQUE INDEX IF NOT EXISTS idx_code_snippets_shortcut
    ON code_snippets(shortcut) WHERE shortcut IS NOT NULL AND shortcut != '';
CREATE INDEX IF NOT EXISTS idx_code_snippets_meta_group
    ON code_snippets(json_extract(meta, '$.group'));
