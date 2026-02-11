//! Rule evaluation engine.
//!
//! This module applies the static SEBI rule catalog to extracted `Signals`
//! and artifact metadata, producing a list of deterministically ordered
//! triggered rules.
//!
//! Responsibilities:
//! - Evaluate rule trigger conditions
//! - Attach structured evidence
//! - Preserve catalog metadata
//! - Guarantee deterministic ordering
//!
//! Non-responsibilities:
//! - Parsing WASM
//! - Classifying overall risk level
//! - Mutating signals
//! - Performing probabilistic inference

use serde_json::json;

use crate::{
    rules::catalog::{RuleId, Severity, catalog},
    signals::model::Signals,
    util::deterministic,
    wasm::parse::ParseConfig,
    wasm::read::ArtifactContext,
};

/// A rule that has been triggered after evaluating signals.
///
/// This struct is purely interpretive and contains:
/// - the rule identity
/// - fixed severity from catalog
/// - static rule metadata
/// - structured evidence derived from signals
///
/// Evidence must reference schema-defined fields only.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TriggeredRule {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub title: String,
    pub message: String,
    pub evidence: serde_json::Value,
}

/// Applies the SEBI rule catalog to the provided signals.
///
/// Returns a deterministically sorted list of triggered rules.
///
/// Determinism guarantees:
/// - Same inputs → identical rule set
/// - Output order sorted by `RuleId`
/// - No hidden randomness
pub fn evaluate_rules(
    signals: &Signals,
    artifact: &ArtifactContext,
    cfg: &ParseConfig,
) -> Vec<TriggeredRule> {
    let mut out = Vec::new();

    for def in catalog() {
        match def.id {
            RuleId::RMem01 => {
                if !signals.memory.has_max {
                    out.push(build_trigger(
                        def,
                        json!({
                            "signals.memory.has_max": signals.memory.has_max,
                            "signals.memory.min_pages": signals.memory.min_pages,
                        }),
                    ));
                }
            }

            RuleId::RMem02 => {
                if signals.instructions.has_memory_grow {
                    out.push(build_trigger(def, json!({
                        "signals.instructions.has_memory_grow": signals.instructions.has_memory_grow,
                        "signals.instructions.memory_grow_count": signals.instructions.memory_grow_count,
                    })));
                }
            }

            RuleId::RCall01 => {
                if signals.instructions.has_call_indirect {
                    out.push(build_trigger(def, json!({
                        "signals.instructions.has_call_indirect": signals.instructions.has_call_indirect,
                        "signals.instructions.call_indirect_count": signals.instructions.call_indirect_count,
                    })));
                }
            }

            RuleId::RLoop01 => {
                if signals.instructions.has_loop {
                    out.push(build_trigger(
                        def,
                        json!({
                            "signals.instructions.has_loop": signals.instructions.has_loop,
                            "signals.instructions.loop_count": signals.instructions.loop_count,
                        }),
                    ));
                }
            }

            RuleId::RSize01 => {
                if artifact.size_bytes > cfg.size_threshold_bytes {
                    out.push(build_trigger(
                        def,
                        json!({
                            "artifact.size_bytes": artifact.size_bytes,
                            "SIZE_THRESHOLD": cfg.size_threshold_bytes,
                        }),
                    ));
                }
            }
        }
    }

    deterministic::sort_triggered_rules(&mut out);
    out
}

/// Helper to construct a `TriggeredRule` from a `RuleDef`.
fn build_trigger(
    def: crate::rules::catalog::RuleDef,
    evidence: serde_json::Value,
) -> TriggeredRule {
    TriggeredRule {
        rule_id: def.id,
        severity: def.severity,
        title: def.title.to_string(),
        message: def.message.to_string(),
        evidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::model::*;
    use crate::wasm::read::ArtifactContext;

    fn base_signals() -> Signals {
        Signals {
            module: ModuleSignals {
                function_count: 0,
                section_count: None,
            },
            memory: MemorySignals {
                memory_count: 1,
                min_pages: Some(1),
                max_pages: Some(10),
                has_max: true,
            },
            imports_exports: ImportExportSignals {
                import_count: 0,
                export_count: 0,
                imports: Some(vec![]),
                exports: Some(vec![]),
            },
            instructions: InstructionSignals {
                has_memory_grow: false,
                memory_grow_count: 0,
                has_call_indirect: false,
                call_indirect_count: 0,
                has_loop: false,
                loop_count: 0,
            },
        }
    }

    fn artifact(size: u64) -> ArtifactContext {
        ArtifactContext {
            path: None,
            bytes: vec![],
            size_bytes: size,
            hash_alg: "sha256".into(),
            hash_hex: "00".into(),
        }
    }

    fn cfg() -> ParseConfig {
        ParseConfig {
            size_threshold_bytes: 100,
        }
    }

    #[test]
    fn triggers_memory_missing_max() {
        let mut s = base_signals();
        s.memory.has_max = false;

        let rules = evaluate_rules(&s, &artifact(10), &cfg());

        assert!(rules.iter().any(|r| r.rule_id == RuleId::RMem01));
    }

    #[test]
    fn triggers_memory_grow() {
        let mut s = base_signals();
        s.instructions.has_memory_grow = true;

        let rules = evaluate_rules(&s, &artifact(10), &cfg());

        assert!(rules.iter().any(|r| r.rule_id == RuleId::RMem02));
    }

    #[test]
    fn triggers_call_indirect() {
        let mut s = base_signals();
        s.instructions.has_call_indirect = true;

        let rules = evaluate_rules(&s, &artifact(10), &cfg());

        assert!(rules.iter().any(|r| r.rule_id == RuleId::RCall01));
    }

    #[test]
    fn triggers_loop() {
        let mut s = base_signals();
        s.instructions.has_loop = true;

        let rules = evaluate_rules(&s, &artifact(10), &cfg());

        assert!(rules.iter().any(|r| r.rule_id == RuleId::RLoop01));
    }

    #[test]
    fn triggers_size_rule() {
        let s = base_signals();
        let rules = evaluate_rules(&s, &artifact(1000), &cfg());

        assert!(rules.iter().any(|r| r.rule_id == RuleId::RSize01));
    }

    #[test]
    fn no_rules_triggered_when_clean() {
        let s = base_signals();
        let rules = evaluate_rules(&s, &artifact(10), &cfg());

        assert!(rules.is_empty());
    }

    #[test]
    fn deterministic_output() {
        let mut s = base_signals();
        s.memory.has_max = false;
        s.instructions.has_loop = true;

        let r1 = evaluate_rules(&s, &artifact(10), &cfg());
        let r2 = evaluate_rules(&s, &artifact(10), &cfg());

        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap()
        );
    }
}
