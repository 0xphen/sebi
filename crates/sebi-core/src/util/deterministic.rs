//! Deterministic ordering helpers.
//!
//! These utilities enforce stable ordering guarantees required by the
//! SEBI report schema. All ordering here is semantic and intentional,
//! ensuring identical inputs always produce identical outputs.

use crate::rules::eval::TriggeredRule;
use crate::wasm::sections::{ExportFact, ImportFact};

/// Sort imports deterministically by `(module, name, kind)`.
///
/// This ordering is part of the SEBI schema contract and must not change
/// without a schema version bump.
pub fn sort_imports(imports: &mut [ImportFact]) {
    imports.sort_by(|a, b| {
        (a.module.as_str(), a.name.as_str(), a.kind.as_str()).cmp(&(
            b.module.as_str(),
            b.name.as_str(),
            b.kind.as_str(),
        ))
    });
}

/// Sort exports deterministically by `(name, kind)`.
///
/// Ensures stable JSON output regardless of WASM section order.
pub fn sort_exports(exports: &mut [ExportFact]) {
    exports.sort_by(|a, b| {
        (a.name.as_str(), a.kind.as_str()).cmp(&(b.name.as_str(), b.kind.as_str()))
    });
}

/// Sort triggered rules by `rule_id`.
///
/// Rule ordering is deterministic and independent of evaluation order.
pub fn sort_triggered_rules(rules: &mut [TriggeredRule]) {
    rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::catalog::{RuleId, Severity};
    use crate::rules::eval::TriggeredRule;
    use crate::wasm::sections::{ExportFact, ImportFact};
    use serde_json::json;

    #[test]
    fn sort_imports_orders_by_module_then_name_then_kind() {
        let mut imports = vec![
            ImportFact {
                module: "env".to_string(),
                name: "memory".to_string(),
                kind: "memory".to_string(),
            },
            ImportFact {
                module: "env".to_string(),
                name: "abort".to_string(),
                kind: "func".to_string(),
            },
            ImportFact {
                module: "wasi_snapshot_preview1".to_string(),
                name: "fd_write".to_string(),
                kind: "func".to_string(),
            },
            ImportFact {
                module: "env".to_string(),
                name: "abort".to_string(),
                kind: "global".to_string(),
            },
        ];

        sort_imports(&mut imports);

        let ordered: Vec<(&str, &str, &str)> = imports
            .iter()
            .map(|i| (i.module.as_str(), i.name.as_str(), i.kind.as_str()))
            .collect();

        assert_eq!(
            ordered,
            vec![
                ("env", "abort", "func"),
                ("env", "abort", "global"),
                ("env", "memory", "memory"),
                ("wasi_snapshot_preview1", "fd_write", "func"),
            ]
        );
    }

    #[test]
    fn sort_imports_is_stable_for_identical_entries() {
        let mut imports = vec![
            ImportFact {
                module: "env".to_string(),
                name: "foo".to_string(),
                kind: "func".to_string(),
            },
            ImportFact {
                module: "env".to_string(),
                name: "foo".to_string(),
                kind: "func".to_string(),
            },
        ];

        sort_imports(&mut imports);

        // Stability here means: no panic, no reordering surprises.
        // Exact identity order is preserved because keys are identical.
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].name, "foo");
        assert_eq!(imports[1].name, "foo");
    }

    #[test]
    fn sort_exports_orders_by_name_then_kind() {
        let mut exports = vec![
            ExportFact {
                name: "memory".to_string(),
                kind: "memory".to_string(),
            },
            ExportFact {
                name: "_start".to_string(),
                kind: "func".to_string(),
            },
            ExportFact {
                name: "_start".to_string(),
                kind: "global".to_string(),
            },
        ];

        sort_exports(&mut exports);

        let ordered: Vec<(&str, &str)> = exports
            .iter()
            .map(|e| (e.name.as_str(), e.kind.as_str()))
            .collect();

        assert_eq!(
            ordered,
            vec![
                ("_start", "func"),
                ("_start", "global"),
                ("memory", "memory"),
            ]
        );
    }

    #[test]
    fn sort_triggered_rules_orders_by_rule_id() {
        let mut rules = vec![
            TriggeredRule {
                rule_id: RuleId("R-LOOP-01".to_string()),
                severity: Severity::MED,
                title: "Loop detected".to_string(),
                message: "loop present".to_string(),
                evidence: json!({}),
            },
            TriggeredRule {
                rule_id: RuleId("R-MEM-02".to_string()),
                severity: Severity::HIGH,
                title: "Memory grow".to_string(),
                message: "memory.grow detected".to_string(),
                evidence: json!({}),
            },
            TriggeredRule {
                rule_id: RuleId("R-CALL-01".to_string()),
                severity: Severity::HIGH,
                title: "call_indirect".to_string(),
                message: "dynamic dispatch".to_string(),
                evidence: json!({}),
            },
        ];

        sort_triggered_rules(&mut rules);

        let ids: Vec<&str> = rules.iter().map(|r| r.rule_id.0.as_str()).collect();

        assert_eq!(ids, vec!["R-CALL-01", "R-LOOP-01", "R-MEM-02"]);
    }

    #[test]
    fn sort_triggered_rules_is_deterministic_across_runs() {
        let make_rules = || {
            vec![
                TriggeredRule {
                    rule_id: RuleId("R-MEM-02".to_string()),
                    severity: Severity::HIGH,
                    title: "Memory grow".to_string(),
                    message: "memory.grow detected".to_string(),
                    evidence: json!({}),
                },
                TriggeredRule {
                    rule_id: RuleId("R-MEM-01".to_string()),
                    severity: Severity::MED,
                    title: "Missing max".to_string(),
                    message: "no max".to_string(),
                    evidence: json!({}),
                },
            ]
        };

        let mut first = make_rules();
        let mut second = make_rules();

        sort_triggered_rules(&mut first);
        sort_triggered_rules(&mut second);

        let first_ids: Vec<&str> = first.iter().map(|r| r.rule_id.0.as_str()).collect();
        let second_ids: Vec<&str> = second.iter().map(|r| r.rule_id.0.as_str()).collect();

        assert_eq!(first_ids, second_ids);
    }
}
