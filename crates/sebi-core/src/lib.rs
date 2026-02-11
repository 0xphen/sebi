//! SEBI (Simple Execution Boundary Inspector)
//!
//! Entry point for WASM artifact inspection and risk classification.

pub mod report;
pub mod rules;
pub mod signals;
pub mod util;
pub mod wasm;

use anyhow::Result;
use report::model::{Report, ToolInfo};
use std::path::Path;

/// Primary tool identity.
pub const TOOL_NAME: &str = "SEBI";

/// Schema version for generated JSON reports.
/// Must be bumped when `report::model` changes semantically.
pub const SCHEMA_VERSION: &str = "0.1.0";

/// Version of the authoritative rule catalog.
pub const RULE_CATALOG_VERSION: &str = "0.1.0";

/// Orchestrates the full inspection pipeline for a WASM artifact.
///
/// The pipeline follows a strict linear flow:
/// 1. **Load**: Read file and compute identity hashes.
/// 2. **Parse**: Extract low-level WASM structures and instructions.
/// 3. **Extract**: Transform structures into schema-stable signals.
/// 4. **Evaluate**: Check signals against the rule catalog.
/// 5. **Classify**: Derive a risk verdict and CI exit code.
/// 6. **Report**: Package all context into a final serializable report.
pub fn inspect(path: &Path, tool: ToolInfo) -> Result<Report> {
    let artifact_ctx = wasm::read::read_artifact(path)?;
    let raw = wasm::parse::parse_wasm(&artifact_ctx.bytes)?;
    let signals = signals::extract::extract_signals(&raw.sections, &raw.instructions);
    let triggered = rules::eval::evaluate_rules(&signals, &artifact_ctx, &raw.config);
    let classification = rules::classify::classify(&triggered);

    // Assemble report
    let report = Report::new(
        tool,
        artifact_ctx.into_artifact(),
        signals,
        raw.analysis,
        raw.rules_catalog,
        triggered,
        classification,
    );

    Ok(report)
}