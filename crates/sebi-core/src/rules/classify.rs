//! Classification engine for SEBI rule evaluation.
//!
//! This module derives a final risk verdict from a set of triggered rules.
//!
//! Responsibilities:
//! - Combine rule severities into a single classification level
//! - Apply a transparent, deterministic policy
//! - Compute CI-compatible exit codes
//! - Preserve stable ordering of rule identifiers
//!
//! Non-responsibilities:
//! - Evaluating rule trigger conditions (handled in `rules::eval`)
//! - Parsing WASM artifacts
//! - Mutating signals
//!
//! The classification policy is intentionally simple and explainable:
//!
//!   - Any HIGH severity rule  → HIGH_RISK
//!   - Else any MED severity    → RISK
//!   - Else                     → SAFE
//!
//! This policy is deterministic and does not depend on rule evaluation order.

use crate::report::model::{ClassificationInfo, ClassificationLevel};
use crate::rules::catalog::Severity;
use crate::rules::eval::TriggeredRule;

/// Derives a final classification from triggered rules.
///
/// This function applies the default SEBI classification policy
/// to a list of already-triggered rules.
///
/// Determinism guarantees:
/// - Same `triggered` input → identical `ClassificationInfo`
/// - Rule IDs sorted canonically before inclusion
/// - Exit codes stable and policy-defined
///
/// Exit code mapping:
/// - SAFE      → 0
/// - RISK      → 1
/// - HIGH_RISK → 2
pub fn classify(triggered: &[TriggeredRule]) -> ClassificationInfo {
    // No triggered rules implies SAFE under default policy.
    if triggered.is_empty() {
        return ClassificationInfo::safe("default");
    }

    // Compute the highest observed severity across all triggered rules.
    // Severity ordering is semantic: LOW < MED < HIGH.
    let highest = triggered
        .iter()
        .map(|r| &r.severity)
        .max()
        .cloned()
        .unwrap_or(Severity::Low);

    let level = if triggered.iter().any(|r| r.severity == Severity::High) {
        ClassificationLevel::HighRisk
    } else if triggered.iter().any(|r| r.severity == Severity::Med) {
        ClassificationLevel::Risk
    } else {
        ClassificationLevel::Safe
    };

    // CI-compatible exit code derived strictly from classification level.
    let exit_code = match level {
        ClassificationLevel::Safe => 0,
        ClassificationLevel::Risk => 1,
        ClassificationLevel::HighRisk => 2,
    };

    let mut triggered_rule_ids: Vec<_> = triggered.iter().map(|r| r.rule_id).collect();
    triggered_rule_ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));

    ClassificationInfo {
        level,
        policy: "default".to_string(),
        reason: "classification derived from triggered rules".to_string(),
        highest_severity: format!("{:?}", highest),
        triggered_rule_ids,
        exit_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::catalog::{RuleId, Severity};
    use crate::rules::eval::TriggeredRule;
    use serde_json::json;

    fn tr(id: RuleId, sev: Severity) -> TriggeredRule {
        TriggeredRule {
            rule_id: id,
            severity: sev,
            title: "t".into(),
            message: "m".into(),
            evidence: json!({}),
        }
    }

    #[test]
    fn test_severity_escalation() {
        let triggered = vec![
            tr(RuleId::RMem01, Severity::Low),
            tr(RuleId::RMem02, Severity::High),
            tr(RuleId::RLoop01, Severity::Med),
        ];
        let c = classify(&triggered);
        assert_eq!(c.level, ClassificationLevel::HighRisk);
        assert_eq!(c.highest_severity, "High");
    }

    #[test]
    fn test_deterministic_id_sorting() {
        let triggered = vec![
            tr(RuleId::RSize01, Severity::Low),
            tr(RuleId::RMem01, Severity::Low),
        ];
        let c = classify(&triggered);
        // Assumes RuleId enum order: RMem01 is before RSize01
        assert_eq!(c.triggered_rule_ids, vec![RuleId::RMem01, RuleId::RSize01]);
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(classify(&[]).exit_code, 0);
        assert_eq!(classify(&[tr(RuleId::RMem01, Severity::Med)]).exit_code, 1);
        assert_eq!(classify(&[tr(RuleId::RMem01, Severity::High)]).exit_code, 2);
    }

    #[test]
    fn empty_triggered_rules_is_safe_default() {
        let c = classify(&[]);
        assert_eq!(c.level, ClassificationLevel::Safe);
        assert_eq!(c.policy, "default");
        assert_eq!(c.exit_code, 0);
    }

    #[test]
    fn any_high_results_in_high_risk() {
        let triggered = vec![
            tr(RuleId::RMem01, Severity::Med),
            tr(RuleId::RCall01, Severity::High),
            tr(RuleId::RLoop01, Severity::Med),
        ];
        let c = classify(&triggered);
        assert_eq!(c.level, ClassificationLevel::HighRisk);
        assert_eq!(c.exit_code, 2);
        assert_eq!(c.highest_severity, "High");
    }

    #[test]
    fn med_without_high_results_in_risk() {
        let triggered = vec![
            tr(RuleId::RMem01, Severity::Med),
            tr(RuleId::RLoop01, Severity::Med),
        ];
        let c = classify(&triggered);
        assert_eq!(c.level, ClassificationLevel::Risk);
        assert_eq!(c.exit_code, 1);
        assert_eq!(c.highest_severity, "Med");
    }

    #[test]
    fn only_low_results_in_safe() {
        let triggered = vec![
            tr(RuleId::RMem01, Severity::Low),
            tr(RuleId::RLoop01, Severity::Low),
        ];
        let c = classify(&triggered);
        assert_eq!(c.level, ClassificationLevel::Safe);
        assert_eq!(c.exit_code, 0);
        assert_eq!(c.highest_severity, "Low");
    }

    #[test]
    fn triggered_rule_ids_are_sorted_deterministically() {
        // Deliberately unsorted order
        let triggered = vec![
            tr(RuleId::RMem02, Severity::High),  // R-MEM-02
            tr(RuleId::RCall01, Severity::High), // R-CALL-01
            tr(RuleId::RLoop01, Severity::Med),  // R-LOOP-01
        ];

        let c = classify(&triggered);

        // Sorted by canonical external rule id:
        // R-CALL-01, R-LOOP-01, R-MEM-02
        assert_eq!(
            c.triggered_rule_ids,
            vec![RuleId::RCall01, RuleId::RLoop01, RuleId::RMem02]
        );
    }

    #[test]
    fn classification_is_deterministic_for_same_input() {
        let triggered = vec![
            tr(RuleId::RLoop01, Severity::Med),
            tr(RuleId::RMem01, Severity::Med),
        ];

        let c1 = classify(&triggered);
        let c2 = classify(&triggered);

        assert_eq!(c1, c2);
    }
}
