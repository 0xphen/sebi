//! Deterministic sorting for WASM facts and triggered rules.
//!
//! Enforces stable ordering to ensure identical WASM inputs produce
//! identical JSON report outputs.

use crate::rules::eval::TriggeredRule;
use crate::wasm::sections::{ExportFact, ImportFact};

/// Sorts imports by `(module, name, kind)`.
pub fn sort_imports(imports: &mut [ImportFact]) {
    imports.sort_by(|a, b| {
        a.module
            .cmp(&b.module)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.kind.cmp(&b.kind))
    });
}

/// Sorts exports by `(name, kind)`.
pub fn sort_exports(exports: &mut [ExportFact]) {
    exports.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.kind.cmp(&b.kind)));
}

/// Sorts triggered rules by canonical rule identifier string.
///
/// Ordering is based on the stable external rule ID
/// (e.g. "R-CALL-01", "R-MEM-02") rather than enum
/// discriminant order. This preserves schema-level
/// determinism even if enum variants are reordered.
pub fn sort_triggered_rules(rules: &mut [TriggeredRule]) {
    rules.sort_by(|a, b| a.rule_id.to_string().cmp(&b.rule_id.to_string()));
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
                module: "env".into(),
                name: "memory".into(),
                kind: "memory".into(),
            },
            ImportFact {
                module: "env".into(),
                name: "abort".into(),
                kind: "func".into(),
            },
            ImportFact {
                module: "wasi".into(),
                name: "fd_write".into(),
                kind: "func".into(),
            },
            ImportFact {
                module: "env".into(),
                name: "abort".into(),
                kind: "global".into(),
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
                ("wasi", "fd_write", "func"),
            ]
        );
    }

    #[test]
    fn sort_exports_orders_by_name_then_kind() {
        let mut exports = vec![
            ExportFact {
                name: "memory".into(),
                kind: "memory".into(),
            },
            ExportFact {
                name: "_start".into(),
                kind: "func".into(),
            },
            ExportFact {
                name: "_start".into(),
                kind: "global".into(),
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
    fn sort_triggered_rules_orders_by_enum_variant() {
        let mut rules = vec![
            TriggeredRule {
                rule_id: RuleId::RLoop01,
                severity: Severity::Med,
                title: "Loop".into(),
                message: "loop present".into(),
                evidence: json!({}),
            },
            TriggeredRule {
                rule_id: RuleId::RMem02,
                severity: Severity::High,
                title: "Mem grow".into(),
                message: "memory.grow".into(),
                evidence: json!({}),
            },
            TriggeredRule {
                rule_id: RuleId::RCall01,
                severity: Severity::High,
                title: "Call indirect".into(),
                message: "call_indirect".into(),
                evidence: json!({}),
            },
        ];

        sort_triggered_rules(&mut rules);

        let ids: Vec<RuleId> = rules.iter().map(|r| r.rule_id).collect();

        assert_eq!(ids, vec![RuleId::RCall01, RuleId::RLoop01, RuleId::RMem02,]);
    }

    #[test]
    fn sort_triggered_rules_is_deterministic() {
        let make_rules = || {
            vec![
                TriggeredRule {
                    rule_id: RuleId::RMem02,
                    severity: Severity::High,
                    title: "Mem grow".into(),
                    message: "memory.grow".into(),
                    evidence: json!({}),
                },
                TriggeredRule {
                    rule_id: RuleId::RMem01,
                    severity: Severity::Med,
                    title: "Missing max".into(),
                    message: "no max".into(),
                    evidence: json!({}),
                },
            ]
        };

        let mut first = make_rules();
        let mut second = make_rules();

        sort_triggered_rules(&mut first);
        sort_triggered_rules(&mut second);

        assert_eq!(first, second);
    }
}
