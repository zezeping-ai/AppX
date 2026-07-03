use std::path::{Path, PathBuf};

pub const MAGIC: &[u8; 4] = b"APPX";
pub const VERSION: u8 = 1;
pub const NONCE_LEN: usize = 12;
pub const HEADER_LEN: usize = MAGIC.len() + 1 + NONCE_LEN;

/// AppX 加密命名：`{name}.{lang}.x`，例如 `readme.txt.x`、`app.js.x`
pub fn is_encrypted_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !name.ends_with(".x") {
        return false;
    }
    let inner = &name[..name.len().saturating_sub(2)];
    inner.contains('.')
}

pub fn encrypted_lang_hint(path: &Path) -> Option<String> {
    if !is_encrypted_path(path) {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    let inner = &name[..name.len() - 2];
    let dot = inner.rfind('.')?;
    Some(inner[dot + 1..].to_ascii_lowercase())
}

pub fn plain_extension(path: &Path) -> Option<String> {
    if is_encrypted_path(path) {
        return None;
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
}

pub fn language_from_path(path: &Path) -> String {
    if let Some(hint) = encrypted_lang_hint(path) {
        return map_lang_hint(&hint);
    }
    if let Some(ext) = plain_extension(path) {
        return map_lang_hint(&ext);
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if name == "dockerfile" {
        return "dockerfile".to_string();
    }
    if name == "makefile" {
        return "makefile".to_string();
    }
    if name == ".env" {
        return "ini".to_string();
    }
    if name == ".gitignore" {
        return "plaintext".to_string();
    }
    "plaintext".to_string()
}

fn map_lang_hint(hint: &str) -> String {
    match hint {
        "js" | "mjs" | "cjs" => "javascript".to_string(),
        "ts" | "mts" | "cts" => "typescript".to_string(),
        "tsx" => "typescript".to_string(),
        "jsx" => "javascript".to_string(),
        "mmd" => "markdown".to_string(),
        "java" => "java".to_string(),
        "py" => "python".to_string(),
        "pyw" => "python".to_string(),
        "rs" => "rust".to_string(),
        "go" => "go".to_string(),
        "json" => "json".to_string(),
        "jsonc" => "json".to_string(),
        "webmanifest" => "json".to_string(),
        "html" | "htm" => "html".to_string(),
        "xhtml" => "html".to_string(),
        "css" => "css".to_string(),
        "scss" => "scss".to_string(),
        "less" => "less".to_string(),
        "md" | "markdown" => "markdown".to_string(),
        "xml" => "xml".to_string(),
        "svg" => "xml".to_string(),
        "yaml" | "yml" => "yaml".to_string(),
        "sql" => "sql".to_string(),
        "sh" | "bash" | "zsh" | "fish" => "shell".to_string(),
        "ksh" => "shell".to_string(),
        "ps1" | "psm1" | "psd1" => "powershell".to_string(),
        "bat" | "cmd" => "bat".to_string(),
        "vue" => "html".to_string(),
        "svelte" | "astro" => "html".to_string(),
        "toml" => "toml".to_string(),
        "ini" => "ini".to_string(),
        "cfg" | "conf" | "env" | "properties" => "ini".to_string(),
        "graphql" | "gql" => "graphql".to_string(),
        "proto" => "protobuf".to_string(),
        "r" => "r".to_string(),
        "clj" | "cljs" | "cljc" | "edn" => "clojure".to_string(),
        "ex" | "exs" => "elixir".to_string(),
        "c" | "h" => "c".to_string(),
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => "cpp".to_string(),
        "cs" => "csharp".to_string(),
        "php" | "phtml" => "php".to_string(),
        "rb" | "rake" | "gemspec" => "ruby".to_string(),
        "kt" | "kts" => "kotlin".to_string(),
        "swift" => "swift".to_string(),
        "dart" => "dart".to_string(),
        "lua" => "lua".to_string(),
        "scala" | "sc" => "scala".to_string(),
        "hcl" | "tf" | "tfvars" => "hcl".to_string(),
        "tfstate" => "json".to_string(),
        "sol" => "sol".to_string(),
        "dockerfile" => "dockerfile".to_string(),
        "cls" => "apex".to_string(),
        "trigger" => "apex".to_string(),
        "apex" => "apex".to_string(),
        "cls-meta" => "xml".to_string(),
        "csv" | "log" => "plaintext".to_string(),
        "txt" => "plaintext".to_string(),
        _ => "plaintext".to_string(),
    }
}

const EDITABLE_EXTENSIONS: &[&str] = &[
    "txt", "md", "markdown", "json", "js", "mjs", "cjs", "ts", "tsx", "jsx", "java", "py", "rs",
    "go", "html", "htm", "css", "scss", "less", "xml", "yaml", "yml", "sql", "sh", "bash", "vue",
    "toml", "ini", "cfg", "conf", "env", "c", "h", "cpp", "cc", "cxx", "hpp", "cs", "php", "rb",
    "kt", "kts", "swift", "dart", "lua", "r", "scala", "clj", "ex", "exs", "dockerfile",
    "gitignore", "properties",
];

const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "svg", "pdf", "zip", "gz", "tar", "7z",
    "rar", "dmg", "exe", "dll", "so", "dylib", "wasm", "mp3", "mp4", "mov", "avi", "mkv", "woff",
    "woff2", "ttf", "otf", "eot", "bin", "dat", "lock",
];

pub fn is_editable_path(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    if is_encrypted_path(path) {
        return true;
    }

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    if lower == "dockerfile" || lower == "makefile" || lower == ".gitignore" {
        return true;
    }

    if let Some(ext) = plain_extension(path) {
        if BINARY_EXTENSIONS.contains(&ext.as_str()) {
            return false;
        }
        if EDITABLE_EXTENSIONS.contains(&ext.as_str()) {
            return true;
        }
        return true;
    }

    !name.starts_with('.')
}

pub fn ensure_writable_path(path: &Path) -> Result<(), String> {
    if is_encrypted_path(path) || is_editable_path(path) {
        Ok(())
    } else {
        Err(format!("不支持的文件类型：{}", path.display()))
    }
}

/// `app.js` -> `app.js.x`，`readme.txt` -> `readme.txt.x`
pub fn encrypted_path_from_plain(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("untitled");
    parent.join(format!("{name}.x"))
}

/// `app.js.x` -> `app.js`，`readme.txt.x` -> `readme.txt`
pub fn plain_path_from_encrypted(path: &Path) -> Option<PathBuf> {
    if !is_encrypted_path(path) {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    let plain_name = &name[..name.len() - 2];
    path.parent().map(|parent| parent.join(plain_name))
}
