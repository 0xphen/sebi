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
