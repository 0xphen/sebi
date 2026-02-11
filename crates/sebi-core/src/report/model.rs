use serde::{Deserialize, Serialize};

use crate::rules::catalog::RuleId;

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
    pub status: String,
    pub warnings: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ClassificationLevel {
    Safe,
    Risk,
    HighRisk,
}

impl std::fmt::Display for ClassificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ClassificationLevel::Safe => "SAFE",
            ClassificationLevel::Risk => "RISK",
            ClassificationLevel::HighRisk => "HIGH_RISK",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClassificationInfo {
    pub level: ClassificationLevel,
    pub policy: String,
    pub reason: String,
    pub highest_severity: String,
    pub triggered_rule_ids: Vec<RuleId>,
    pub exit_code: i32,
}

impl ClassificationInfo {
    pub fn safe(policy: &str) -> Self {
        Self {
            level: ClassificationLevel::Safe,
            policy: policy.to_string(),
            reason: "no rules triggered".to_string(),
            highest_severity: "NONE".to_string(),
            triggered_rule_ids: vec![],
            exit_code: 0,
        }
    }
}
