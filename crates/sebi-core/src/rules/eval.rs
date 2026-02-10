use crate::rules::catalog::{RuleId, Severity};

#[derive(Debug, Clone)]
pub struct TriggeredRule {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub title: String,
    pub message: String,
    pub evidence: serde_json::Value,
}
