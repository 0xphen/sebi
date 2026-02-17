#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::NamedTempFile;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn sebi_cmd() -> Command {
    Command::cargo_bin("sebi-cli").expect("binary should be built")
}

#[test]
fn safe_contract_exits_0() {
    sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .assert()
        .code(0);
}

#[test]
fn safe_erc20_exits_0() {
    sebi_cmd()
        .arg(fixtures_dir().join("stylus_erc20_safe.wasm"))
        .assert()
        .code(0);
}

#[test]
fn risk_contract_exits_1() {
    sebi_cmd()
        .arg(fixtures_dir().join("cpp_kv_store_simple.wasm"))
        .assert()
        .code(1);
}

#[test]
fn high_risk_contract_exits_2() {
    sebi_cmd()
        .arg(fixtures_dir().join("rust_registry_complex.wasm"))
        .assert()
        .code(2);
}

#[test]
fn high_risk_cpp_bridge_exits_2() {
    sebi_cmd()
        .arg(fixtures_dir().join("cpp_token_bridge_complex.wasm"))
        .assert()
        .code(2);
}

#[test]
fn high_risk_dex_router_exits_2() {
    sebi_cmd()
        .arg(fixtures_dir().join("stylus_dex_router_complex.wasm"))
        .assert()
        .code(2);
}

#[test]
fn json_output_is_valid() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .arg("--format")
        .arg("json")
        .output()
        .expect("command should run");

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");

    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("tool").is_some());
    assert!(parsed.get("artifact").is_some());
    assert!(parsed.get("signals").is_some());
    assert!(parsed.get("analysis").is_some());
    assert!(parsed.get("rules").is_some());
    assert!(parsed.get("classification").is_some());
}

#[test]
fn json_classification_safe_for_safe_contract() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .output()
        .expect("command should run");

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["classification"]["level"], "SAFE");
    assert_eq!(parsed["classification"]["exit_code"], 0);
    assert!(parsed["rules"]["triggered"].as_array().unwrap().is_empty());
}

#[test]
fn json_classification_risk_for_loop_contract() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("cpp_kv_store_simple.wasm"))
        .output()
        .expect("command should run");

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["classification"]["level"], "RISK");
    assert_eq!(parsed["classification"]["exit_code"], 1);

    let triggered: Vec<&str> = parsed["rules"]["triggered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["rule_id"].as_str().unwrap())
        .collect();
    assert!(triggered.contains(&"R-LOOP-01"));
}

#[test]
fn json_classification_high_risk_for_complex_contract() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_registry_complex.wasm"))
        .output()
        .expect("command should run");

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["classification"]["level"], "HIGH_RISK");
    assert_eq!(parsed["classification"]["exit_code"], 2);

    let triggered: Vec<&str> = parsed["rules"]["triggered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["rule_id"].as_str().unwrap())
        .collect();
    assert!(triggered.contains(&"R-MEM-01"));
    assert!(triggered.contains(&"R-MEM-02"));
    assert!(triggered.contains(&"R-CALL-01"));
    assert!(triggered.contains(&"R-LOOP-01"));
}

#[test]
fn json_schema_version_present() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .output()
        .expect("command should run");

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["schema_version"], "0.1.0");
}

#[test]
fn json_tool_info_reflects_binary() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .output()
        .expect("command should run");

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["tool"]["name"], "sebi-cli");
    assert_eq!(parsed["tool"]["version"], "0.1.0");
    assert!(parsed["tool"]["commit"].is_null());
}

#[test]
fn json_artifact_has_hash() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .output()
        .expect("command should run");

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["artifact"]["hash"]["algorithm"], "sha256");
    let hash = parsed["artifact"]["hash"]["value"].as_str().unwrap();
    assert_eq!(hash.len(), 64, "SHA-256 hex should be 64 chars");
}

#[test]
fn text_output_contains_classification() {
    sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .arg("--format")
        .arg("text")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Classification: Safe"));
}

#[test]
fn text_output_shows_triggered_rules() {
    sebi_cmd()
        .arg(fixtures_dir().join("rust_registry_complex.wasm"))
        .arg("--format")
        .arg("text")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("R-MEM-01"))
        .stdout(predicate::str::contains("R-MEM-02"))
        .stdout(predicate::str::contains("R-CALL-01"))
        .stdout(predicate::str::contains("R-LOOP-01"));
}

#[test]
fn out_flag_writes_to_file() {
    let tmp = NamedTempFile::new().expect("create temp file");
    let out_path = tmp.path().to_path_buf();

    sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .arg("--out")
        .arg(&out_path)
        .assert()
        .code(0)
        .stdout(predicate::str::is_empty());

    let contents = std::fs::read_to_string(&out_path).expect("read output file");
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("file should be JSON");
    assert_eq!(parsed["classification"]["level"], "SAFE");
}

#[test]
fn out_flag_with_text_format() {
    let tmp = NamedTempFile::new().expect("create temp file");
    let out_path = tmp.path().to_path_buf();

    sebi_cmd()
        .arg(fixtures_dir().join("cpp_kv_store_simple.wasm"))
        .arg("--format")
        .arg("text")
        .arg("--out")
        .arg(&out_path)
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());

    let contents = std::fs::read_to_string(&out_path).expect("read output file");
    assert!(contents.contains("Classification:"));
    assert!(contents.contains("R-LOOP-01"));
}

#[test]
fn commit_flag_embeds_hash_in_report() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .arg("--commit")
        .arg("abc123def456")
        .output()
        .expect("command should run");

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["tool"]["commit"], "abc123def456");
}

#[test]
fn no_commit_flag_leaves_null() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .output()
        .expect("command should run");

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(parsed["tool"]["commit"].is_null());
}

#[test]
fn missing_wasm_arg_fails() {
    sebi_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn nonexistent_file_fails() {
    sebi_cmd()
        .arg("/tmp/does_not_exist_sebi_test.wasm")
        .assert()
        .failure();
}

#[test]
fn invalid_format_flag_fails() {
    sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .arg("--format")
        .arg("xml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn deterministic_json_across_runs() {
    let fixture = fixtures_dir().join("cpp_token_bridge_complex.wasm");

    let output_a = sebi_cmd().arg(&fixture).output().expect("first run");

    let output_b = sebi_cmd().arg(&fixture).output().expect("second run");

    let json_a: serde_json::Value = serde_json::from_slice(&output_a.stdout).unwrap();
    let json_b: serde_json::Value = serde_json::from_slice(&output_b.stdout).unwrap();

    // Compare everything except artifact.path (may differ in representation)
    assert_eq!(json_a["schema_version"], json_b["schema_version"]);
    assert_eq!(json_a["signals"], json_b["signals"]);
    assert_eq!(json_a["rules"], json_b["rules"]);
    assert_eq!(json_a["classification"], json_b["classification"]);
    assert_eq!(json_a["artifact"]["hash"], json_b["artifact"]["hash"]);
    assert_eq!(
        json_a["artifact"]["size_bytes"],
        json_b["artifact"]["size_bytes"]
    );
}

#[test]
fn help_flag_prints_usage() {
    sebi_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Static execution-boundary inspection",
        ));
}

#[test]
fn version_flag_prints_version() {
    sebi_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("sebi"));
}

#[test]
fn default_format_is_json() {
    let output = sebi_cmd()
        .arg(fixtures_dir().join("rust_counter_safe.wasm"))
        .output()
        .expect("command should run");

    // Should parse as JSON without explicit --format json
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("default output should be valid JSON");
}
