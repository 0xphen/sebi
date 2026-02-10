//! SEBI rule catalog.
//!
//! Defines the authoritative, static set of execution-boundary rules used by
//! SEBI to interpret extracted WASM signals into risk classifications.
//!
//! This module is intentionally declarative:
//! - no WASM parsing
//! - no instruction inspection
//! - no runtime inference
//!
//! Rules operate only on schema-defined signals and are evaluated by
//! `rules::eval`.

use serde::{Deserialize, Serialize};

/// Stable identifier for a rule.
///
/// Rule IDs are globally unique, stable across releases,
/// and never reused once published.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuleId(pub String);

/// Fixed severity level assigned to a rule.
///
/// Ordering is semantic and relied upon by classification logic:
/// `LOW < MED < HIGH`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    LOW,
    MED,
    HIGH,
}

/// Static metadata describing a SEBI rule.
///
/// Contains no trigger logic or evaluation state.
/// Rule evaluation is performed by mapping signals to these definitions.
#[derive(Debug, Clone)]
pub struct RuleDef {
    /// Unique rule identifier (e.g. `R-MEM-01`)
    pub id: RuleId,

    /// Severity associated with the rule
    pub severity: Severity,

    /// Short human-readable title
    pub title: &'static str,

    /// Explanation emitted when the rule is triggered
    pub message: &'static str,
}

/// Returns the complete SEBI rule catalog.
///
/// The catalog is deterministic and immutable.
/// Changes to rule semantics or identifiers require
/// explicit review and a catalog version bump.
pub fn catalog() -> Vec<RuleDef> {
    vec![
        RuleDef {
            id: RuleId("R-MEM-01".to_string()),
            severity: Severity::MED,
            title: "Missing declared memory maximum",
            message: "Memory has no declared maximum; static bounding is reduced.",
        },
        RuleDef {
            id: RuleId("R-MEM-02".to_string()),
            severity: Severity::HIGH,
            title: "Runtime memory growth detected",
            message: "memory.grow present; runtime memory expansion capability detected.",
        },
        RuleDef {
            id: RuleId("R-CALL-01".to_string()),
            severity: Severity::HIGH,
            title: "Dynamic dispatch via function tables",
            message: "call_indirect present; dynamic dispatch reduces static call-graph predictability.",
        },
        RuleDef {
            id: RuleId("R-LOOP-01".to_string()),
            severity: Severity::MED,
            title: "Loop constructs detected",
            message: "loop present; termination cannot always be proven statically.",
        },
        RuleDef {
            id: RuleId("R-SIZE-01".to_string()),
            severity: Severity::MED,
            title: "Large WASM artifact",
            message: "Artifact size exceeds threshold; complexity correlation signal.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Ensures rule identifiers remain globally unique.
    #[test]
    fn rule_ids_are_unique() {
        let rules = catalog();
        let mut seen = HashSet::new();

        for rule in rules {
            assert!(
                seen.insert(rule.id.0.clone()),
                "duplicate rule id detected: {}",
                rule.id.0
            );
        }
    }

    /// Locks in the intended severity ordering.
    #[test]
    fn severity_ordering_is_low_to_high() {
        assert!(Severity::LOW < Severity::MED);
        assert!(Severity::MED < Severity::HIGH);
    }
}
