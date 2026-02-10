use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuleId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    LOW,
    MED,
    HIGH,
}

#[derive(Debug, Clone)]
pub struct RuleDef {
    pub id: RuleId,
    pub severity: Severity,
    pub title: &'static str,
    pub message: &'static str,
}

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

    #[test]
    fn severity_ordering_is_low_to_high() {
        assert!(Severity::LOW < Severity::MED);
        assert!(Severity::MED < Severity::HIGH);
    }
}
