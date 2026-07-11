-- FTS5 全文索引：preview + inline 正文
CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts USING fts5(
    content,
    item_id UNINDEXED,
    tokenize = 'unicode61 remove_diacritics 0'
);

-- 回填已有数据（幂等）
INSERT INTO clipboard_fts(item_id, content)
SELECT ci.id, ci.preview FROM clipboard_items ci
WHERE NOT EXISTS (
    SELECT 1 FROM clipboard_fts f WHERE f.item_id = ci.id AND f.content = ci.preview
);

INSERT INTO clipboard_fts(item_id, content)
SELECT t.item_id, t.content FROM clipboard_text t
WHERE NOT EXISTS (
    SELECT 1 FROM clipboard_fts f WHERE f.item_id = t.item_id AND f.content = t.content
);

CREATE TRIGGER IF NOT EXISTS clipboard_items_ai AFTER INSERT ON clipboard_items BEGIN
    INSERT INTO clipboard_fts(item_id, content) VALUES (new.id, new.preview);
END;

CREATE TRIGGER IF NOT EXISTS clipboard_items_ad AFTER DELETE ON clipboard_items BEGIN
    DELETE FROM clipboard_fts WHERE item_id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS clipboard_items_au AFTER UPDATE OF preview ON clipboard_items BEGIN
    DELETE FROM clipboard_fts WHERE item_id = old.id;
    INSERT INTO clipboard_fts(item_id, content) VALUES (new.id, new.preview);
END;

CREATE TRIGGER IF NOT EXISTS clipboard_text_ai AFTER INSERT ON clipboard_text BEGIN
    INSERT INTO clipboard_fts(item_id, content) VALUES (new.item_id, new.content);
END;

CREATE TRIGGER IF NOT EXISTS clipboard_text_ad AFTER DELETE ON clipboard_text BEGIN
    DELETE FROM clipboard_fts WHERE item_id = old.item_id AND content = old.content;
END;

CREATE TRIGGER IF NOT EXISTS clipboard_text_au AFTER UPDATE OF content ON clipboard_text BEGIN
    DELETE FROM clipboard_fts WHERE item_id = old.item_id AND content = old.content;
    INSERT INTO clipboard_fts(item_id, content) VALUES (new.item_id, new.content);
END;
