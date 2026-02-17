use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION;
use crate::rules::catalog::RuleId;
use crate::rules::eval::TriggeredRule;
use crate::signals::model::Signals;

/// Top-level SEBI report.
///
/// This struct is the stable JSON contract defined in `SCHEMA.md`.
/// It must remain deterministic for identical input artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: String,
    pub tool: ToolInfo,
    pub artifact: ArtifactInfo,
    pub signals: Signals,
    pub analysis: AnalysisInfo,
    pub rules: RulesInfo,
    pub classification: ClassificationInfo,
}

impl Report {
    /// Construct a new SEBI report from pipeline outputs.
    ///
    /// Assumes `triggered` rules are already deterministically sorted.
    pub fn new(
        tool: ToolInfo,
        artifact: ArtifactInfo,
        signals: Signals,
        analysis: AnalysisInfo,
        catalog: RulesCatalogInfo,
        mut triggered: Vec<TriggeredRule>,
        mut classification: ClassificationInfo,
    ) -> Self {
        triggered.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));

        let triggered_rule_ids: Vec<RuleId> = triggered.iter().map(|r| r.rule_id).collect();

        let rules = RulesInfo {
            catalog,
            triggered: triggered
                .into_iter()
                .map(|r| TriggeredRuleInfo {
                    rule_id: r.rule_id.to_string(),
                    severity: format!("{:?}", r.severity),
                    title: r.title,
                    message: r.message,
                    evidence: r.evidence,
                })
                .collect(),
        };

        classification.triggered_rule_ids = triggered_rule_ids;

        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            tool,
            artifact,
            signals,
            analysis,
            rules,
            classification,
        }
    }
}

/// Tool metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub commit: Option<String>,
}

/// Artifact metadata bound to this report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub path: Option<String>,
    pub size_bytes: u64,
    pub hash: ArtifactHash,
}

/// Cryptographic artifact fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactHash {
    pub algorithm: String,
    pub value: String,
}

/// Parsing/analysis status.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisInfo {
    pub status: String,
    pub warnings: Vec<String>,
}

impl AnalysisInfo {
    pub fn ok() -> Self {
        Self {
            status: "ok".into(),
            warnings: vec![],
        }
    }

    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self {
            status: "parse_error".into(),
            warnings: vec![msg.into()],
        }
    }

    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self {
            status: "unsupported".into(),
            warnings: vec![msg.into()],
        }
    }
}

/// Rule evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesInfo {
    pub catalog: RulesCatalogInfo,
    pub triggered: Vec<TriggeredRuleInfo>,
}

/// Rule catalog metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RulesCatalogInfo {
    pub catalog_version: String,
    pub ruleset: String,
}

/// Triggered rule entry included in report output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredRuleInfo {
    pub rule_id: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub evidence: serde_json::Value,
}

/// Final classification level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClassificationLevel {
    Safe,
    Risk,
    HighRisk,
}

impl std::fmt::Display for ClassificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().trim_matches('"')
        )
    }
}

/// Final classification block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
            policy: policy.into(),
            reason: "no rules triggered".into(),
            highest_severity: "NONE".into(),
            triggered_rule_ids: vec![],
            exit_code: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::catalog::{RuleId, Severity};
    use crate::rules::eval::TriggeredRule;
    use serde_json::json;

    fn dummy_signals() -> Signals {
        Signals {
            module: Default::default(),
            memory: Default::default(),
            imports_exports: Default::default(),
            instructions: Default::default(),
        }
    }

    fn dummy_triggered() -> Vec<TriggeredRule> {
        vec![TriggeredRule {
            rule_id: RuleId::RMem01,
            severity: Severity::Med,
            title: "Missing memory max".into(),
            message: "Memory has no declared maximum.".into(),
            evidence: json!({"has_max": false}),
        }]
    }

    #[test]
    fn report_maps_triggered_rules_and_ids() {
        let report = Report::new(
            ToolInfo {
                name: "sebi".into(),
                version: "1.0.0".into(),
                commit: None,
            },
            ArtifactInfo {
                path: None,
                size_bytes: 123,
                hash: ArtifactHash {
                    algorithm: "sha256".into(),
                    value: "abc".into(),
                },
            },
            dummy_signals(),
            AnalysisInfo::ok(),
            RulesCatalogInfo {
                catalog_version: "0.1.0".into(),
                ruleset: "default".into(),
            },
            dummy_triggered(),
            ClassificationInfo::safe("default"),
        );

        assert_eq!(report.rules.triggered.len(), 1);
        assert_eq!(report.rules.triggered[0].rule_id, "R-MEM-01");

        assert_eq!(
            report.classification.triggered_rule_ids,
            vec![RuleId::RMem01]
        );
    }

    #[test]
    fn analysis_info_factories() {
        let err = AnalysisInfo::parse_error("failed");
        assert_eq!(err.status, "parse_error");
        assert_eq!(err.warnings, vec!["failed"]);

        let ok = AnalysisInfo::ok();
        assert_eq!(ok.status, "ok");
        assert!(ok.warnings.is_empty());
    }

    #[test]
    fn classification_serializes_correctly() {
        let level = ClassificationLevel::HighRisk;
        let serialized = serde_json::to_string(&level).unwrap();

        // Must match SCHEMA.md: "HIGH_RISK"
        assert_eq!(serialized, "\"HIGH_RISK\"");
    }
}
