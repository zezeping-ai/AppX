-- 按应用覆盖的内容变体：[{ "appId", "appName", "content"(密文) }, ...]
ALTER TABLE code_snippets ADD COLUMN app_variants TEXT NOT NULL DEFAULT '[]';
