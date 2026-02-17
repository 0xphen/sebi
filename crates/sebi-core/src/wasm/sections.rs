//! Section-level fact extraction for SEBI.
//!
//! This module performs a **structural scan of WASM sections** and extracts
//! *observable facts* required by higher layers (signals and rules).
//!
//! Key design principles:
//! - No interpretation or policy logic
//! - No instruction-level analysis
//! - Deterministic output
//! - Mirrors WASM semantics precisely
//!
//! The extracted data feeds into:
//! - signal extraction
//! - rule evaluation
//! - final risk classification

use crate::util::deterministic;
use anyhow::Result;
use wasmparser::{
    Export, ExportSectionReader, ExternalKind, FunctionSectionReader, ImportSectionReader,
    MemorySectionReader, MemoryType, TableSectionReader, TypeRef,
};

/// Aggregated facts derived from WASM *sections*.
///
/// This struct represents a **lossless but minimal summary** of module structure.
/// All fields are derived directly from the binary without inference.
///
/// Invariants:
/// - Counts are saturating and monotonic
/// - Memory limits are derived from memory index 0 only
/// - Imports and exports are sorted deterministically
#[derive(Debug, Clone, Default)]
pub struct SectionFacts {
    /// Total number of imports declared
    pub import_count: u32,

    /// Total number of exports declared
    pub export_count: u32,

    /// Number of *defined* functions (from the Function section)
    pub function_count: u32,

    /// Whether a table section exists (any table)
    pub has_table_section: bool,

    /// Total number of memory declarations (imported + internal)
    pub memory_count: u32,

    /// Minimum pages of memory index 0, if present
    pub memory_min_pages: Option<u64>,

    /// Maximum pages of memory index 0, if declared
    pub memory_max_pages: Option<u64>,

    /// Convenience flag: true iff memory index 0 declares a maximum
    pub memory_has_max: bool,

    /// Normalized list of import facts
    pub imports: Vec<ImportFact>,

    /// Normalized list of export facts
    pub exports: Vec<ExportFact>,
}

/// Normalized representation of a single import.
///
/// This intentionally stores only:
/// - module name
/// - import name
/// - external kind
///
/// Type signatures and indices are handled elsewhere.
#[derive(Debug, Clone)]
pub struct ImportFact {
    pub module: String,
    pub name: String,
    pub kind: String, // "func" | "memory" | "table" | "global" | "tag"
}

/// Normalized representation of a single export.
#[derive(Debug, Clone)]
pub struct ExportFact {
    pub name: String,
    pub kind: String,
}

/// Processes the Import section and records import-related facts.
///
/// This function:
/// - supports all WASM import encodings (Single / Compact1 / Compact2)
/// - flattens grouped imports
/// - records memory imports consistently
/// - enforces deterministic ordering
pub fn on_import_section(facts: &mut SectionFacts, reader: ImportSectionReader) -> Result<()> {
    facts.import_count = facts.import_count.saturating_add(reader.count());

    for item in reader {
        let imports_group: wasmparser::Imports = item?;

        match imports_group {
            wasmparser::Imports::Single(_, imp) => {
                process_single_import(facts, imp.module, imp.name, imp.ty);
            }

            wasmparser::Imports::Compact1 { module, items } => {
                for inner in items {
                    let compact = inner?;
                    process_single_import(facts, module, compact.name, compact.ty);
                }
            }

            wasmparser::Imports::Compact2 { module, ty, names } => {
                for name_item in names {
                    let name = name_item?;
                    process_single_import(facts, module, name, ty);
                }
            }
        }
    }

    deterministic::sort_imports(&mut facts.imports);
    Ok(())
}

/// Processes the Memory section.
///
/// Notes:
/// - All memory declarations are counted
/// - Only memory index 0 determines limits (per WASM semantics)
/// - Multi-memory is supported without ambiguity
pub fn on_memory_section(facts: &mut SectionFacts, reader: MemorySectionReader) -> Result<()> {
    let count = reader.count();
    facts.memory_count = facts.memory_count.saturating_add(count);

    // Only the first memory sets execution-boundary limits
    for (i, item) in reader.into_iter().enumerate() {
        let mem = item?;
        if i == 0 {
            record_first_memory_limits(facts, &mem);
        }
    }

    Ok(())
}

/// Records a single import entry in normalized form.
///
/// This function centralizes:
/// - kind mapping
/// - memory detection
/// - memory limit propagation
fn process_single_import(facts: &mut SectionFacts, module: &str, name: &str, ty: TypeRef) {
    let (kind_str, maybe_mem) = match ty {
        TypeRef::Func(_) | TypeRef::FuncExact(_) => ("func", None),
        TypeRef::Table(_) => ("table", None),
        TypeRef::Global(_) => ("global", None),
        TypeRef::Tag(_) => ("tag", None),
        TypeRef::Memory(mem) => ("memory", Some(mem)),
    };

    facts.imports.push(ImportFact {
        module: module.to_string(),
        name: name.to_string(),
        kind: kind_str.to_string(),
    });

    // Imported memory contributes to total memory count
    if let Some(mem) = maybe_mem {
        facts.memory_count = facts.memory_count.saturating_add(1);
        record_first_memory_limits(facts, &mem);
    }
}

/// Processes the Export section.
///
/// This function:
/// - records export name and external kind
/// - normalizes kind strings
/// - enforces deterministic ordering
pub fn on_export_section(facts: &mut SectionFacts, reader: ExportSectionReader) -> Result<()> {
    facts.export_count = facts.export_count.saturating_add(reader.count());

    for item in reader {
        let ex: Export = item?;
        facts.exports.push(ExportFact {
            name: ex.name.to_string(),
            kind: export_kind_str(ex.kind),
        });
    }

    deterministic::sort_exports(&mut facts.exports);
    Ok(())
}

pub fn on_function_section(facts: &mut SectionFacts, reader: FunctionSectionReader) -> Result<()> {
    facts.function_count = facts.function_count.saturating_add(reader.count());
    Ok(())
}

/// Processes the Table section.
///
/// Presence alone is sufficient for execution-boundary reasoning.
pub fn on_table_section(facts: &mut SectionFacts, _reader: TableSectionReader) -> Result<()> {
    facts.has_table_section = true;
    Ok(())
}

/// Records memory limits for memory index 0.
///
/// This function is idempotent and will not overwrite existing limits.
fn record_first_memory_limits(facts: &mut SectionFacts, mem: &MemoryType) {
    if facts.memory_min_pages.is_none() {
        facts.memory_min_pages = Some(mem.initial);
        facts.memory_max_pages = mem.maximum;
        facts.memory_has_max = mem.maximum.is_some();
    }
}

/// Maps WASM external kinds into stable string identifiers.
fn export_kind_str(k: ExternalKind) -> String {
    match k {
        ExternalKind::Func | ExternalKind::FuncExact => "func",
        ExternalKind::Table => "table",
        ExternalKind::Memory => "memory",
        ExternalKind::Global => "global",
        ExternalKind::Tag => "tag",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmparser::{Parser, Payload};

    fn parse_wasm(wat: &str) -> SectionFacts {
        let wasm = wat::parse_str(wat).expect("Failed to parse WAT");
        let mut facts = SectionFacts::default();

        for payload in Parser::new(0).parse_all(&wasm) {
            match payload.expect("Parser error") {
                Payload::ImportSection(r) => on_import_section(&mut facts, r).unwrap(),
                Payload::ExportSection(r) => on_export_section(&mut facts, r).unwrap(),
                Payload::MemorySection(r) => on_memory_section(&mut facts, r).unwrap(),
                Payload::FunctionSection(r) => on_function_section(&mut facts, r).unwrap(),
                Payload::TableSection(r) => on_table_section(&mut facts, r).unwrap(),
                _ => {}
            }
        }
        facts
    }

    #[test]
    fn test_memory_precedence_import_vs_internal() {
        let facts = parse_wasm(
            r#"
            (module
              (import "env" "mem" (memory 1 3))
              (memory 5 10)
            )
            "#,
        );

        assert_eq!(facts.memory_min_pages, Some(1));
        assert_eq!(facts.memory_max_pages, Some(3));
        assert!(facts.memory_has_max);
        assert_eq!(facts.memory_count, 2);
    }

    #[test]
    fn test_compact_imports_variants() {
        let facts = parse_wasm(
            r#"
            (module
              (import "env" "f1" (func))
              (import "env" "f2" (func))
              (import "env" "m1" (memory 1))
              (import "os" "exit" (func))
            )
        "#,
        );

        assert_eq!(facts.import_count, 4);
        assert_eq!(facts.memory_count, 1);

        let names: Vec<String> = facts.imports.iter().map(|i| i.name.clone()).collect();
        assert_eq!(names, vec!["f1", "f2", "m1", "exit"]); // "env" module items come before "os"
    }

    #[test]
    fn test_no_memory_declared() {
        let facts = parse_wasm("(module (func))");

        assert_eq!(facts.memory_count, 0);
        assert!(facts.memory_min_pages.is_none());
        assert!(!facts.memory_has_max);
    }

    #[test]
    fn test_deterministic_export_sorting() {
        let facts = parse_wasm(
            r#"
            (module
              (func $f)
              (export "z_export" (func $f))
              (export "a_export" (func $f))
            )
            "#,
        );

        assert_eq!(facts.exports[0].name, "a_export");
        assert_eq!(facts.exports[1].name, "z_export");
    }

    #[test]
    fn test_unbounded_memory_detection() {
        let facts = parse_wasm(r#"(module (memory 1))"#);

        assert_eq!(facts.memory_min_pages, Some(1));
        assert!(facts.memory_max_pages.is_none());
        assert!(!facts.memory_has_max);
    }

    #[test]
    fn test_multi_memory_feature_limits() {
        let facts = parse_wasm(
            r#"
            (module
              (memory 1 2)
              (memory 3 4)
            )
            "#,
        );

        assert_eq!(facts.memory_count, 2);
        assert_eq!(facts.memory_min_pages, Some(1));
        assert_eq!(facts.memory_max_pages, Some(2));
    }

    #[test]
    fn test_export_kind_mapping() {
        let facts = parse_wasm(
            r#"
            (module
              (func $f)
              (table $t 1 funcref)
              (memory $m 1)
              (global $g (mut i32) (i32.const 0))
              (export "e_func" (func $f))
              (export "e_table" (table $t))
              (export "e_mem" (memory $m))
              (export "e_global" (global $g))
            )
        "#,
        );

        let kinds: Vec<(String, String)> = facts
            .exports
            .iter()
            .map(|e| (e.name.clone(), e.kind.clone()))
            .collect();

        // Sorted by name: e_func, e_global, e_mem, e_table
        assert_eq!(kinds[0], ("e_func".to_string(), "func".to_string()));
        assert_eq!(kinds[1], ("e_global".to_string(), "global".to_string()));
        assert_eq!(kinds[2], ("e_mem".to_string(), "memory".to_string()));
        assert_eq!(kinds[3], ("e_table".to_string(), "table".to_string()));
    }

    #[test]
    fn test_empty_module_invariants() {
        let facts = parse_wasm(r#"(module)"#);

        assert_eq!(facts.import_count, 0);
        assert_eq!(facts.export_count, 0);
        assert_eq!(facts.function_count, 0);
        assert_eq!(facts.memory_count, 0);
        assert!(facts.memory_min_pages.is_none());
        assert!(!facts.has_table_section);
    }
}
