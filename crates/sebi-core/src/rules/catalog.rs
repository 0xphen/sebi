//! SEBI rule catalog defining risk classifications for WASM signals.
//!
//! This module is strictly declarative and contains no evaluation logic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Low,
    Med,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuleId {
    RMem01,
    RMem02,
    RCall01,
    RLoop01,
    RSize01,
}

impl RuleId {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleId::RMem01 => "R-MEM-01",
            RuleId::RMem02 => "R-MEM-02",
            RuleId::RCall01 => "R-CALL-01",
            RuleId::RLoop01 => "R-LOOP-01",
            RuleId::RSize01 => "R-SIZE-01",
        }
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RuleId::RMem01 => "R-MEM-01",
            RuleId::RMem02 => "R-MEM-02",
            RuleId::RCall01 => "R-CALL-01",
            RuleId::RLoop01 => "R-LOOP-01",
            RuleId::RSize01 => "R-SIZE-01",
        };
        write!(f, "{s}")
    }
}

/// Static metadata for a risk classification rule.
#[derive(Debug, Clone)]
pub struct RuleDef {
    pub id: RuleId,
    pub severity: Severity,
    pub title: &'static str,
    pub message: &'static str,
}

/// Returns the immutable SEBI rule catalog.
pub fn catalog() -> Vec<RuleDef> {
    vec![
        RuleDef {
            id: RuleId::RMem01,
            severity: Severity::Med,
            title: "Missing declared memory maximum",
            message: "Memory has no declared maximum; static bounding is reduced.",
        },
        RuleDef {
            id: RuleId::RMem02,
            severity: Severity::High,
            title: "Runtime memory growth detected",
            message: "memory.grow present; runtime memory expansion capability detected.",
        },
        RuleDef {
            id: RuleId::RCall01,
            severity: Severity::High,
            title: "Dynamic dispatch via function tables",
            message: "call_indirect present; dynamic dispatch reduces call-graph predictability.",
        },
        RuleDef {
            id: RuleId::RLoop01,
            severity: Severity::Med,
            title: "Loop constructs detected",
            message: "loop present; termination cannot always be proven statically.",
        },
        RuleDef {
            id: RuleId::RSize01,
            severity: Severity::Med,
            title: "Large WASM artifact",
            message: "Artifact size exceeds threshold; complexity correlation signal.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn rule_ids_are_unique() {
        let rules = catalog();
        let mut seen = HashSet::new();

        for rule in rules {
            assert!(
                seen.insert(rule.id),
                "Duplicate rule id detected: {:?}",
                rule.id
            );
        }
    }

    #[test]
    fn severity_ordering_is_correct() {
        assert!(Severity::Low < Severity::Med);
        assert!(Severity::Med < Severity::High);
    }
}
