use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizeResult {
    pub path: String,
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub saved_bytes: i64,
    pub status: OptimizeStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OptimizeStatus {
    Optimized,
    AlreadyOptimal,
    Error,
}

impl OptimizeResult {
    pub fn optimized(path: String, before: u64, after: u64) -> Self {
        Self {
            path,
            before_bytes: before,
            after_bytes: after,
            saved_bytes: before as i64 - after as i64,
            status: OptimizeStatus::Optimized,
            error: None,
        }
    }

    pub fn already_optimal(path: String, before: u64) -> Self {
        Self {
            path,
            before_bytes: before,
            after_bytes: before,
            saved_bytes: 0,
            status: OptimizeStatus::AlreadyOptimal,
            error: None,
        }
    }

    pub fn failed(path: String, before: u64, error: impl Into<String>) -> Self {
        Self {
            path,
            before_bytes: before,
            after_bytes: before,
            saved_bytes: 0,
            status: OptimizeStatus::Error,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizeFileInput {
    pub path: String,
    /// 缺省则使用磁盘设置中的 lossyEnabled
    #[serde(default)]
    pub lossy: Option<bool>,
}
