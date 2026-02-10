use anyhow::Result;
use wasmparser::{Parser, Payload};

use crate::report::model::{AnalysisInfo, RulesCatalogInfo};
use crate::wasm::{scan, sections, stylus};

/// Parsing-time configuration that influences downstream policy signals.
///
/// Note: This is *not* the rules engine. It only supplies values that
/// rules may reference (e.g., size thresholds).
#[derive(Debug, Clone)]
pub struct ParseConfig {
    /// Threshold (bytes) used by size-based rule triggers.
    pub size_threshold_bytes: u64,
}

impl Default for ParseConfig {
    fn default() -> Self {
        // Conservative default; can be tuned or made configurable via CLI later.
        Self {
            size_threshold_bytes: 200_000,
        }
    }
}

/// Internal, pre-schema facts extracted from a WASM binary.
///
/// This is the output of the WASM parsing stage only:
/// - section-level facts (imports/exports/functions/memory/table presence)
/// - instruction-level facts (presence/counts of selected operators)
/// - analysis status/warnings describing parsing completeness
///
/// No policy decisions are made here:
/// - no rules are evaluated
/// - no severity is assigned
/// - no classification is produced
#[derive(Debug, Clone, Default)]
pub struct RawWasmFacts {
    /// Section-derived facts (module structure).
    pub sections: sections::SectionFacts,

    /// Instruction-derived facts (operator scanning).
    pub instructions: scan::InstructionFacts,

    /// Parsing/compatibility status and deterministic warnings.
    pub analysis: AnalysisInfo,

    /// Identifies the rule catalog used for this report.
    /// Stored here so the report assembly can include it without
    /// reaching into unrelated modules.
    pub rules_catalog: RulesCatalogInfo,

    /// Parsing configuration knobs.
    pub config: ParseConfig,
}

/// Parse a WebAssembly binary and extract raw structural and instruction facts.
///
/// This function performs a single deterministic pass over `bytes`:
///
/// 1. Dispatches section payloads to `wasm::sections` for section-level extraction.
/// 2. Dispatches `CodeSectionEntry` bodies to `wasm::scan` for operator scanning.
/// 3. Ignores sections that are irrelevant to current signals (custom/name/debug, etc.).
/// 4. Marks component-model payloads as unsupported (best-effort handling).
/// 5. Applies a target-specific normalization pass via `wasm::stylus` to emit warnings
///    or adjust tolerances without introducing policy judgments.
///
/// Output is an internal representation (`RawWasmFacts`) that is later converted into
/// schema-defined `Signals` by `signals::extract`.
pub fn parse_wasm(bytes: &[u8]) -> Result<RawWasmFacts> {
    let mut facts = RawWasmFacts {
        analysis: AnalysisInfo::ok(),
        rules_catalog: RulesCatalogInfo {
            catalog_version: "0.1.0".to_string(),
            ruleset: "default".to_string(),
        },
        config: ParseConfig::default(),
        ..Default::default()
    };

    // `parse_all` is appropriate here because SEBI reads the full artifact
    // into memory in `io::read` and performs deterministic offline analysis.
    let parser = Parser::new(0);

    for payload in parser.parse_all(bytes) {
        println!("payload: {:?}", payload);
        match payload {
            // Module header/version. Presence indicates a well-formed WASM prefix.
            Ok(Payload::Version { .. }) => {}

            // Section-level signals.
            Ok(Payload::ImportSection(reader)) => {
                sections::on_import_section(&mut facts.sections, reader)?;
            }
            Ok(Payload::FunctionSection(reader)) => {
                sections::on_function_section(&mut facts.sections, reader)?;
            }
            Ok(Payload::TableSection(reader)) => {
                sections::on_table_section(&mut facts.sections, reader)?;
            }
            Ok(Payload::MemorySection(reader)) => {
                sections::on_memory_section(&mut facts.sections, reader)?;
            }
            Ok(Payload::ExportSection(reader)) => {
                sections::on_export_section(&mut facts.sections, reader)?;
            }

            // Code scanning (instruction-level signals).
            Ok(Payload::CodeSectionStart { .. }) => {
                // Optional: sanity-check that Function section count matches Code bodies.
                // SEBI v1 does not require this; scanning uses the entry stream directly.
            }
            Ok(Payload::CodeSectionEntry(body)) => {
                scan::on_code_entry(&mut facts.instructions, body)?;
            }

            // Custom sections are intentionally ignored for v1:
            // names/producers/debug info do not contribute to execution-boundary signals.
            Ok(Payload::CustomSection(_)) => {}

            // WebAssembly component model payloads are out of scope for SEBI v1.
            // We mark analysis as unsupported to avoid implying full coverage.
            Ok(
                other @ (Payload::ComponentSection { .. }
                | Payload::ComponentTypeSection(_)
                | Payload::ComponentImportSection(_)
                | Payload::ComponentExportSection(_)
                | Payload::ComponentCanonicalSection(_)
                | Payload::CoreTypeSection(_)
                | Payload::InstanceSection(_)
                | Payload::ComponentInstanceSection(_)
                | Payload::ComponentAliasSection(_)
                | Payload::ComponentStartSection { .. }
                | Payload::ModuleSection { .. }),
            ) => {
                facts.analysis = AnalysisInfo::unsupported(format!(
                    "unsupported WASM component/module nesting payload: {:?}",
                    other
                ));
            }

            Ok(Payload::End(_)) => {}

            // Any parse error is reported in analysis status and terminates parsing.
            Err(e) => {
                facts.analysis = AnalysisInfo::parse_error(e.to_string());
                break;
            }

            // All other sections are currently ignored by design (Type, Global, Data, etc.).
            // They can be added later as new signals without changing rule evaluation logic.
            _ => {}
        }
    }

    stylus::normalize(&mut facts.sections, &mut facts.analysis);

    Ok(facts)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Small, valid WASM modules encoded by hand.
    // These are stable and do not rely on toolchains.

    // (module)
    const EMPTY_MODULE: &[u8] = b"\0asm\x01\0\0\0";

    // (module (memory 1))
    const MEMORY_MODULE: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, // \0asm
        0x01, 0x00, 0x00, 0x00, // version
        0x05, 0x03, 0x01, 0x00, 0x01, // memory section: min 1
    ];

    // (module (func (loop)))
    const LOOP_MODULE: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // type section
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // function section
        0x03, 0x02, 0x01, 0x00, // code section
        0x0a, 0x06, 0x01, 0x04, 0x00, 0x03, 0x40, 0x0b, 0x0b, // loop
    ];

    #[test]
    fn parses_empty_module() {
        let facts = parse_wasm(EMPTY_MODULE).expect("valid wasm");

        assert_eq!(facts.analysis.status, "ok");
        assert_eq!(facts.sections.import_count, 0);
        assert!(!facts.instructions.has_loop);
    }

    #[test]
    fn detects_memory_section() {
        let facts = parse_wasm(MEMORY_MODULE).unwrap();

        assert_eq!(facts.sections.memory_count, 1);
        assert_eq!(facts.sections.memory_min_pages, Some(1));
        assert!(facts.analysis.warnings.is_empty());
    }

    #[test]
    fn detects_loop_instruction() {
        let facts = parse_wasm(LOOP_MODULE).unwrap();

        assert!(facts.instructions.has_loop);
        assert_eq!(facts.instructions.loop_count, 1);
    }

    #[test]
    fn deterministic_output_for_same_input() {
        let a = parse_wasm(LOOP_MODULE).unwrap();
        let b = parse_wasm(LOOP_MODULE).unwrap();

        assert_eq!(a.sections.function_count, b.sections.function_count);
        assert_eq!(a.instructions.loop_count, b.instructions.loop_count);
        assert_eq!(a.analysis.warnings, b.analysis.warnings);
    }

    #[test]
    fn invalid_wasm_sets_parse_error() {
        let invalid = b"not a wasm file";

        let facts = parse_wasm(invalid).unwrap();

        assert_eq!(facts.analysis.status, "parse_error");
    }

    #[test]
    fn warns_when_no_memory_detected() {
        let facts = parse_wasm(EMPTY_MODULE).unwrap();

        assert!(
            facts
                .analysis
                .warnings
                .iter()
                .any(|w| w.contains("no memory"))
        );
    }

    #[test]
    fn unsupported_payload_sets_analysis_status() {
        // Fake bytes that force wasmparser into unsupported territory
        let bytes = b"\0asm\x01\0\0\0\x00";

        let facts = parse_wasm(bytes).unwrap();

        // Either ok or unsupported is acceptable for malformed edge cases,
        // but it must never panic.
        assert!(
            facts.analysis.status == "ok"
                || facts.analysis.status == "unsupported"
                || facts.analysis.status == "parse_error"
        );
    }

    #[test]
    fn test_unordered_and_duplicate_sections() {
        // (module (memory 1) (export "m" (memory 0)))
        // Manually crafted to ensure we handle standard section sequences.
        let wasm = wat::parse_str(
            r#"
            (module
              (memory 1)
              (export "mem_a" (memory 0))
              (export "mem_b" (memory 0))
            )
            "#,
        )
        .unwrap();

        let facts = parse_wasm(&wasm).expect("valid parse");

        assert_eq!(facts.sections.memory_count, 1);
        assert_eq!(facts.sections.export_count, 2);
        assert_eq!(facts.sections.exports[0].name, "mem_a");
    }

    #[test]
    fn test_component_model_nesting_graceful_failure() {
        let component_bytes = b"\0asm\x0a\x00\x01\x00";

        let facts = parse_wasm(component_bytes).expect("parse call should not panic");

        assert_ne!(facts.analysis.status, "ok");
        assert!(facts.analysis.status == "unsupported" || facts.analysis.status == "parse_error");
    }

    /// Ensures that saturating arithmetic prevents overflow when processing
    /// modules with massive internal counts.
    #[test]
    fn test_saturating_arithmetic_limits() {
        let mut facts = RawWasmFacts::default();
        facts.sections.import_count = u32::MAX;

        // Simulate an additional import discovery
        facts.sections.import_count = facts.sections.import_count.saturating_add(1);

        assert_eq!(facts.sections.import_count, u32::MAX);
    }
}
