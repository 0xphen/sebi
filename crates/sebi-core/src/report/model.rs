use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub path: Option<String>,
    pub size_bytes: u64,
    pub hash: ArtifactHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactHash {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisInfo {
    pub status: String,        // ok|parse_error|unsupported|partial
    pub warnings: Vec<String>, // sorted
}

impl AnalysisInfo {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            warnings: vec![],
        }
    }
    pub fn parse_error(msg: String) -> Self {
        Self {
            status: "parse_error".to_string(),
            warnings: vec![msg],
        }
    }
    pub fn unsupported(msg: String) -> Self {
        Self {
            status: "unsupported".to_string(),
            warnings: vec![msg],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RulesCatalogInfo {
    pub catalog_version: String,
    pub ruleset: String,
}
