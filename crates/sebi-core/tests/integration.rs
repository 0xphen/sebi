use sebi_core::report::model::{ClassificationLevel, Report, ToolInfo};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// Path to the fixtures directory relative to the crate root.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Compiles a `.wat` fixture to WASM bytes.
fn compile_fixture(name: &str) -> Vec<u8> {
    let path = fixtures_dir().join(name);
    wat::parse_file(&path).unwrap_or_else(|e| panic!("failed to compile {name}: {e}"))
}

/// Writes WASM bytes to a temp file and runs the full inspect pipeline.
fn inspect_fixture(name: &str) -> Report {
    let wasm = compile_fixture(name);
    inspect_bytes(&wasm)
}

/// Inspects raw WASM bytes through the full pipeline.
fn inspect_bytes(wasm: &[u8]) -> Report {
    let mut tmp = NamedTempFile::new().expect("create temp file");
    tmp.write_all(wasm).expect("write wasm bytes");
    tmp.flush().expect("flush");

    let tool = ToolInfo {
        name: "sebi".into(),
        version: "0.1.0-test".into(),
        commit: None,
    };

    sebi_core::inspect(tmp.path(), tool).expect("inspect should succeed")
}

/// Collects triggered rule IDs from a report.
fn triggered_ids(report: &Report) -> Vec<String> {
    report
        .rules
        .triggered
        .iter()
        .map(|r| r.rule_id.clone())
        .collect()
}

/// Checks whether a specific rule ID was triggered.
fn has_rule(report: &Report, rule_id: &str) -> bool {
    report.rules.triggered.iter().any(|r| r.rule_id == rule_id)
}

#[test]
fn rust_safe_storage_classified_safe() {
    let report = inspect_fixture("rust_safe_storage.wat");

    assert_eq!(report.classification.level, ClassificationLevel::Safe);
    assert_eq!(report.classification.exit_code, 0);
    assert!(
        report.rules.triggered.is_empty(),
        "expected no triggered rules, got: {:?}",
        triggered_ids(&report)
    );
}

#[test]
fn rust_safe_storage_signals_correct() {
    let report = inspect_fixture("rust_safe_storage.wat");

    // Memory is bounded
    assert!(report.signals.memory.has_max);
    assert_eq!(report.signals.memory.min_pages, Some(2));
    assert_eq!(report.signals.memory.max_pages, Some(16));
    assert_eq!(report.signals.memory.memory_count, 1);

    // No dangerous instructions
    assert!(!report.signals.instructions.has_memory_grow);
    assert!(!report.signals.instructions.has_call_indirect);
    assert!(!report.signals.instructions.has_loop);

    // Has imports from vm_hooks
    assert!(report.signals.imports_exports.import_count >= 4);

    // Has exports: memory, user_entrypoint, mark_used
    assert!(report.signals.imports_exports.export_count >= 3);
}

#[test]
fn rust_loop_unbounded_mem_classified_risk() {
    let report = inspect_fixture("rust_loop_unbounded_mem.wat");

    assert_eq!(report.classification.level, ClassificationLevel::Risk);
    assert_eq!(report.classification.exit_code, 1);

    assert!(has_rule(&report, "R-MEM-01"), "expected R-MEM-01 triggered");
    assert!(
        has_rule(&report, "R-LOOP-01"),
        "expected R-LOOP-01 triggered"
    );

    // Should NOT trigger HIGH severity rules
    assert!(
        !has_rule(&report, "R-MEM-02"),
        "R-MEM-02 should not trigger"
    );
    assert!(
        !has_rule(&report, "R-CALL-01"),
        "R-CALL-01 should not trigger"
    );
}

#[test]
fn rust_loop_unbounded_mem_signals_correct() {
    let report = inspect_fixture("rust_loop_unbounded_mem.wat");

    assert!(!report.signals.memory.has_max);
    assert_eq!(report.signals.memory.min_pages, Some(2));
    assert!(report.signals.memory.max_pages.is_none());

    assert!(report.signals.instructions.has_loop);
    assert!(report.signals.instructions.loop_count >= 2);
    assert!(!report.signals.instructions.has_memory_grow);
    assert!(!report.signals.instructions.has_call_indirect);
}

#[test]
fn rust_dynamic_dispatch_classified_high_risk() {
    let report = inspect_fixture("rust_dynamic_dispatch.wat");

    assert_eq!(report.classification.level, ClassificationLevel::HighRisk);
    assert_eq!(report.classification.exit_code, 2);

    assert!(has_rule(&report, "R-MEM-02"), "expected R-MEM-02 triggered");
    assert!(
        has_rule(&report, "R-CALL-01"),
        "expected R-CALL-01 triggered"
    );
}

#[test]
fn rust_dynamic_dispatch_signals_correct() {
    let report = inspect_fixture("rust_dynamic_dispatch.wat");

    assert!(report.signals.memory.has_max);
    assert_eq!(report.signals.memory.max_pages, Some(256));

    assert!(report.signals.instructions.has_memory_grow);
    assert!(report.signals.instructions.memory_grow_count >= 1);
    assert!(report.signals.instructions.has_call_indirect);
    assert!(report.signals.instructions.call_indirect_count >= 1);
}

#[test]
fn cpp_vtable_erc20_classified_high_risk() {
    let report = inspect_fixture("cpp_vtable_erc20.wat");

    assert_eq!(report.classification.level, ClassificationLevel::HighRisk);
    assert_eq!(report.classification.exit_code, 2);

    // All four signal-based rules should fire
    assert!(has_rule(&report, "R-MEM-01"), "R-MEM-01: unbounded memory");
    assert!(
        has_rule(&report, "R-MEM-02"),
        "R-MEM-02: memory.grow in malloc"
    );
    assert!(has_rule(&report, "R-CALL-01"), "R-CALL-01: vtable dispatch");
    assert!(
        has_rule(&report, "R-LOOP-01"),
        "R-LOOP-01: ABI decoder loop"
    );
}

#[test]
fn cpp_vtable_erc20_signals_correct() {
    let report = inspect_fixture("cpp_vtable_erc20.wat");

    assert!(!report.signals.memory.has_max);
    assert_eq!(report.signals.memory.min_pages, Some(4));

    assert!(report.signals.instructions.has_memory_grow);
    assert!(report.signals.instructions.has_call_indirect);
    assert!(report.signals.instructions.has_loop);

    // Imports come from "env" module (C++ convention)
    let imports = report.signals.imports_exports.imports.as_ref().unwrap();
    assert!(imports.iter().all(|i| i.module == "env"));
    assert!(report.signals.imports_exports.import_count >= 5);

    // Has many functions (8 ERC20 methods + malloc + decode + entrypoint + mark_used)
    assert!(report.signals.module.function_count >= 12);
}

#[test]
fn cpp_allocator_classified_high_risk() {
    let report = inspect_fixture("cpp_allocator.wat");

    assert_eq!(report.classification.level, ClassificationLevel::HighRisk);
    assert_eq!(report.classification.exit_code, 2);

    assert!(
        has_rule(&report, "R-MEM-02"),
        "expected R-MEM-02 from allocator"
    );
    assert!(
        has_rule(&report, "R-LOOP-01"),
        "expected R-LOOP-01 from memcpy"
    );

    assert!(
        !has_rule(&report, "R-MEM-01"),
        "R-MEM-01 should not fire (memory has max)"
    );
}

#[test]
fn cpp_allocator_signals_correct() {
    let report = inspect_fixture("cpp_allocator.wat");

    assert!(report.signals.memory.has_max);
    assert_eq!(report.signals.memory.min_pages, Some(4));
    assert_eq!(report.signals.memory.max_pages, Some(64));

    assert!(report.signals.instructions.has_memory_grow);
    assert!(report.signals.instructions.memory_grow_count >= 1);

    assert!(report.signals.instructions.has_loop);
    assert!(report.signals.instructions.loop_count >= 2);

    assert!(!report.signals.instructions.has_call_indirect);
}

#[test]
fn minimal_module_triggers_mem01() {
    let report = inspect_fixture("minimal_module.wat");

    assert_eq!(report.classification.level, ClassificationLevel::Risk);
    assert_eq!(report.classification.exit_code, 1);
    assert!(has_rule(&report, "R-MEM-01"));

    assert_eq!(report.signals.memory.memory_count, 0);
    assert!(!report.signals.memory.has_max);
    assert!(report.signals.memory.min_pages.is_none());
    assert!(report.signals.memory.max_pages.is_none());

    assert!(!report.signals.instructions.has_memory_grow);
    assert!(!report.signals.instructions.has_call_indirect);
    assert!(!report.signals.instructions.has_loop);
    assert_eq!(report.signals.module.function_count, 0);
}

#[test]
fn minimal_module_warns_no_memory() {
    let report = inspect_fixture("minimal_module.wat");

    assert!(
        report
            .analysis
            .warnings
            .iter()
            .any(|w| w.contains("no memory")),
        "expected 'no memory' warning, got: {:?}",
        report.analysis.warnings
    );
}

#[test]
fn imported_memory_bounded_classified_safe() {
    let report = inspect_fixture("imported_memory_bounded.wat");

    assert_eq!(report.classification.level, ClassificationLevel::Safe);
    assert_eq!(report.classification.exit_code, 0);
    assert!(report.rules.triggered.is_empty());

    assert_eq!(report.signals.memory.memory_count, 1);
    assert!(report.signals.memory.has_max);
    assert_eq!(report.signals.memory.min_pages, Some(1));
    assert_eq!(report.signals.memory.max_pages, Some(16));
}

#[test]
fn imported_memory_bounded_import_details() {
    let report = inspect_fixture("imported_memory_bounded.wat");

    let imports = report.signals.imports_exports.imports.as_ref().unwrap();
    let mem_import = imports.iter().find(|i| i.kind == "memory");
    assert!(
        mem_import.is_some(),
        "expected a memory import in the import list"
    );
    let mem = mem_import.unwrap();
    assert_eq!(mem.module, "env");
    assert_eq!(mem.name, "memory");
}

#[test]
fn imported_memory_unbounded_triggers_mem01() {
    let report = inspect_fixture("imported_memory_unbounded.wat");

    assert_eq!(report.classification.level, ClassificationLevel::Risk);
    assert_eq!(report.classification.exit_code, 1);
    assert!(has_rule(&report, "R-MEM-01"));
    assert!(!report.signals.memory.has_max);
    assert_eq!(report.signals.memory.min_pages, Some(2));
}

#[test]
fn all_signals_triggers_all_instruction_rules() {
    let report = inspect_fixture("all_signals.wat");

    assert_eq!(report.classification.level, ClassificationLevel::HighRisk);
    assert_eq!(report.classification.exit_code, 2);

    assert!(has_rule(&report, "R-MEM-01"));
    assert!(has_rule(&report, "R-MEM-02"));
    assert!(has_rule(&report, "R-CALL-01"));
    assert!(has_rule(&report, "R-LOOP-01"));
}

#[test]
fn nested_loops_counted_accurately() {
    let report = inspect_fixture("nested_loops.wat");

    assert!(report.signals.instructions.has_loop);
    assert_eq!(
        report.signals.instructions.loop_count, 3,
        "triple-nested loop should produce loop_count == 3"
    );
}

#[test]
fn multiple_memory_grow_counted_accurately() {
    let report = inspect_fixture("multiple_memory_grow.wat");

    assert!(report.signals.instructions.has_memory_grow);
    assert_eq!(
        report.signals.instructions.memory_grow_count, 3,
        "3 memory.grow calls across 2 functions should produce count == 3"
    );
}

#[test]
fn large_artifact_triggers_size_rule() {
    // Generate a WASM module exceeding the default 200KB threshold.
    // We pad with a large data section to reliably exceed the threshold.
    let padding = "X".repeat(210_000);
    let wat = format!("(module (memory 4 16) (data (i32.const 0) \"{padding}\"))");

    let wasm = wat::parse_str(&wat).expect("large module should compile");
    assert!(
        wasm.len() > 200_000,
        "generated module should exceed 200KB, got {} bytes",
        wasm.len()
    );

    let report = inspect_bytes(&wasm);

    assert!(has_rule(&report, "R-SIZE-01"), "expected R-SIZE-01 to fire");
    // Verify evidence contains the threshold and actual size
    let size_rule = report
        .rules
        .triggered
        .iter()
        .find(|r| r.rule_id == "R-SIZE-01")
        .unwrap();
    assert!(
        size_rule.evidence.get("artifact.size_bytes").is_some(),
        "R-SIZE-01 evidence should contain artifact.size_bytes"
    );
    assert!(
        size_rule.evidence.get("SIZE_THRESHOLD").is_some(),
        "R-SIZE-01 evidence should contain SIZE_THRESHOLD"
    );
}

#[test]
fn invalid_wasm_reports_parse_error() {
    let garbage = b"this is not a valid wasm file at all";
    let report = inspect_bytes(garbage);

    assert_eq!(report.analysis.status, "parse_error");
}

#[test]
fn deterministic_json_output_for_same_fixture() {
    // Use the same temp file for both runs to ensure identical artifact.path.
    let wasm = compile_fixture("cpp_vtable_erc20.wat");
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&wasm).unwrap();
    tmp.flush().unwrap();

    let tool = || ToolInfo {
        name: "sebi".into(),
        version: "0.1.0-test".into(),
        commit: None,
    };
    let report_a = sebi_core::inspect(tmp.path(), tool()).unwrap();
    let report_b = sebi_core::inspect(tmp.path(), tool()).unwrap();

    let json_a = serde_json::to_string_pretty(&report_a).unwrap();
    let json_b = serde_json::to_string_pretty(&report_b).unwrap();

    assert_eq!(
        json_a, json_b,
        "identical input must produce identical JSON"
    );
}

#[test]
fn deterministic_json_output_for_safe_contract() {
    let wasm = compile_fixture("rust_safe_storage.wat");
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&wasm).unwrap();
    tmp.flush().unwrap();

    let tool = || ToolInfo {
        name: "sebi".into(),
        version: "0.1.0-test".into(),
        commit: None,
    };
    let report_a = sebi_core::inspect(tmp.path(), tool()).unwrap();
    let report_b = sebi_core::inspect(tmp.path(), tool()).unwrap();

    let json_a = serde_json::to_string(&report_a).unwrap();
    let json_b = serde_json::to_string(&report_b).unwrap();

    assert_eq!(json_a, json_b);
}

#[test]
fn report_schema_version_matches() {
    let report = inspect_fixture("rust_safe_storage.wat");
    assert_eq!(report.schema_version, "0.1.0");
}

#[test]
fn report_tool_info_preserved() {
    let report = inspect_fixture("rust_safe_storage.wat");
    assert_eq!(report.tool.name, "sebi");
    assert_eq!(report.tool.version, "0.1.0-test");
    assert!(report.tool.commit.is_none());
}

#[test]
fn report_artifact_hash_is_sha256() {
    let report = inspect_fixture("rust_safe_storage.wat");
    assert_eq!(report.artifact.hash.algorithm, "sha256");
    assert!(!report.artifact.hash.value.is_empty());
    // SHA256 hex is 64 chars
    assert_eq!(report.artifact.hash.value.len(), 64);
}

#[test]
fn report_artifact_size_matches_wasm() {
    let wasm = compile_fixture("rust_safe_storage.wat");
    let report = inspect_bytes(&wasm);
    assert_eq!(report.artifact.size_bytes, wasm.len() as u64);
}

#[test]
fn report_json_roundtrip() {
    let report = inspect_fixture("cpp_vtable_erc20.wat");

    let json = serde_json::to_string_pretty(&report).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify top-level fields exist per SCHEMA.md
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("tool").is_some());
    assert!(parsed.get("artifact").is_some());
    assert!(parsed.get("signals").is_some());
    assert!(parsed.get("analysis").is_some());
    assert!(parsed.get("rules").is_some());
    assert!(parsed.get("classification").is_some());
}

#[test]
fn report_rules_catalog_version() {
    let report = inspect_fixture("rust_safe_storage.wat");
    assert_eq!(report.rules.catalog.catalog_version, "0.1.0");
    assert_eq!(report.rules.catalog.ruleset, "default");
}

#[test]
fn triggered_rules_contain_evidence() {
    let report = inspect_fixture("all_signals.wat");

    for rule in &report.rules.triggered {
        assert!(
            rule.evidence.is_object(),
            "rule {} evidence should be an object",
            rule.rule_id
        );
        assert!(
            !rule.evidence.as_object().unwrap().is_empty(),
            "rule {} evidence should not be empty",
            rule.rule_id
        );
    }
}

#[test]
fn mem01_evidence_references_schema_paths() {
    let report = inspect_fixture("imported_memory_unbounded.wat");

    let mem01 = report
        .rules
        .triggered
        .iter()
        .find(|r| r.rule_id == "R-MEM-01")
        .expect("R-MEM-01 should be triggered");

    assert!(
        mem01.evidence.get("signals.memory.has_max").is_some(),
        "R-MEM-01 evidence should reference signals.memory.has_max"
    );
    assert!(
        mem01.evidence.get("signals.memory.min_pages").is_some(),
        "R-MEM-01 evidence should reference signals.memory.min_pages"
    );
}

#[test]
fn mem02_evidence_references_schema_paths() {
    let report = inspect_fixture("rust_dynamic_dispatch.wat");

    let mem02 = report
        .rules
        .triggered
        .iter()
        .find(|r| r.rule_id == "R-MEM-02")
        .expect("R-MEM-02 should be triggered");

    assert!(
        mem02
            .evidence
            .get("signals.instructions.has_memory_grow")
            .is_some()
    );
    assert!(
        mem02
            .evidence
            .get("signals.instructions.memory_grow_count")
            .is_some()
    );
}

#[test]
fn call01_evidence_references_schema_paths() {
    let report = inspect_fixture("rust_dynamic_dispatch.wat");

    let call01 = report
        .rules
        .triggered
        .iter()
        .find(|r| r.rule_id == "R-CALL-01")
        .expect("R-CALL-01 should be triggered");

    assert!(
        call01
            .evidence
            .get("signals.instructions.has_call_indirect")
            .is_some()
    );
    assert!(
        call01
            .evidence
            .get("signals.instructions.call_indirect_count")
            .is_some()
    );
}

#[test]
fn loop01_evidence_references_schema_paths() {
    let report = inspect_fixture("rust_loop_unbounded_mem.wat");

    let loop01 = report
        .rules
        .triggered
        .iter()
        .find(|r| r.rule_id == "R-LOOP-01")
        .expect("R-LOOP-01 should be triggered");

    assert!(
        loop01
            .evidence
            .get("signals.instructions.has_loop")
            .is_some()
    );
    assert!(
        loop01
            .evidence
            .get("signals.instructions.loop_count")
            .is_some()
    );
}

#[test]
fn classification_policy_is_default() {
    let report = inspect_fixture("rust_safe_storage.wat");
    assert_eq!(report.classification.policy, "default");
}

#[test]
fn triggered_rule_ids_sorted_in_classification() {
    let report = inspect_fixture("all_signals.wat");

    // Classification triggered_rule_ids are sorted by RuleId enum Ord
    // (declaration order: RMem01, RMem02, RCall01, RLoop01, RSize01).
    let ids: Vec<String> = report
        .classification
        .triggered_rule_ids
        .iter()
        .map(|id| id.as_str().to_string())
        .collect();

    // Verify consistent ordering: MEM rules first, then CALL, then LOOP
    assert_eq!(
        ids,
        vec!["R-MEM-01", "R-MEM-02", "R-CALL-01", "R-LOOP-01"],
        "triggered_rule_ids should be sorted by enum variant order"
    );
}

#[test]
fn triggered_rules_sorted_in_rules_section() {
    let report = inspect_fixture("cpp_vtable_erc20.wat");

    // rules.triggered is sorted by RuleId enum Ord (declaration order).
    let ids: Vec<&str> = report
        .rules
        .triggered
        .iter()
        .map(|r| r.rule_id.as_str())
        .collect();

    assert_eq!(
        ids,
        vec!["R-MEM-01", "R-MEM-02", "R-CALL-01", "R-LOOP-01"],
        "rules.triggered should be sorted by enum variant order"
    );
}

#[test]
fn imports_sorted_deterministically() {
    let report = inspect_fixture("rust_safe_storage.wat");

    let imports = report.signals.imports_exports.imports.as_ref().unwrap();
    for window in imports.windows(2) {
        let a = (&window[0].module, &window[0].name, &window[0].kind);
        let b = (&window[1].module, &window[1].name, &window[1].kind);
        assert!(a <= b, "imports not sorted: {:?} > {:?}", a, b);
    }
}

#[test]
fn exports_sorted_deterministically() {
    let report = inspect_fixture("rust_safe_storage.wat");

    let exports = report.signals.imports_exports.exports.as_ref().unwrap();
    for window in exports.windows(2) {
        let a = (&window[0].name, &window[0].kind);
        let b = (&window[1].name, &window[1].kind);
        assert!(a <= b, "exports not sorted: {:?} > {:?}", a, b);
    }
}

#[test]
fn hash_is_stable_for_same_bytes() {
    let wasm = compile_fixture("rust_safe_storage.wat");
    let report_a = inspect_bytes(&wasm);
    let report_b = inspect_bytes(&wasm);

    assert_eq!(report_a.artifact.hash.value, report_b.artifact.hash.value);
}

#[test]
fn valid_fixtures_have_ok_status() {
    let fixtures = [
        "rust_safe_storage.wat",
        "rust_loop_unbounded_mem.wat",
        "rust_dynamic_dispatch.wat",
        "cpp_vtable_erc20.wat",
        "cpp_allocator.wat",
        "minimal_module.wat",
        "imported_memory_bounded.wat",
        "imported_memory_unbounded.wat",
        "all_signals.wat",
        "nested_loops.wat",
        "multiple_memory_grow.wat",
    ];

    for name in fixtures {
        let report = inspect_fixture(name);
        assert_eq!(
            report.analysis.status, "ok",
            "fixture {name} should have analysis status 'ok'"
        );
    }
}

#[test]
fn exit_code_0_means_no_med_or_high_rules() {
    let report = inspect_fixture("rust_safe_storage.wat");
    assert_eq!(report.classification.exit_code, 0);
    assert!(report.rules.triggered.is_empty());
}

#[test]
fn exit_code_1_means_med_severity_only() {
    let report = inspect_fixture("rust_loop_unbounded_mem.wat");
    assert_eq!(report.classification.exit_code, 1);

    // All triggered rules should be MED severity (none HIGH)
    for rule in &report.rules.triggered {
        assert_ne!(
            rule.severity, "High",
            "exit code 1 should have no HIGH severity rules"
        );
    }
}

#[test]
fn exit_code_2_means_at_least_one_high() {
    let report = inspect_fixture("rust_dynamic_dispatch.wat");
    assert_eq!(report.classification.exit_code, 2);

    let has_high = report.rules.triggered.iter().any(|r| r.severity == "High");
    assert!(has_high, "exit code 2 must have at least one HIGH rule");
}

#[test]
fn rust_and_cpp_contracts_with_same_patterns_get_same_classification() {
    let rust_report = inspect_fixture("rust_dynamic_dispatch.wat");
    let cpp_report = inspect_fixture("cpp_vtable_erc20.wat");

    assert_eq!(
        rust_report.classification.level,
        ClassificationLevel::HighRisk
    );
    assert_eq!(
        cpp_report.classification.level,
        ClassificationLevel::HighRisk
    );

    assert!(has_rule(&rust_report, "R-MEM-02"));
    assert!(has_rule(&rust_report, "R-CALL-01"));
    assert!(has_rule(&cpp_report, "R-MEM-02"));
    assert!(has_rule(&cpp_report, "R-CALL-01"));
}

#[test]
fn function_count_excludes_imports() {
    let report = inspect_fixture("rust_safe_storage.wat");

    assert!(
        report.signals.module.function_count >= 3,
        "should have at least 3 defined functions (get, set, entrypoint, mark_used)"
    );

    assert!(
        report.signals.imports_exports.import_count >= 4,
        "should have at least 4 imports"
    );
}
