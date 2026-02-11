pub mod report;
pub mod rules;
pub mod signals;
pub mod util;
pub mod wasm;

pub const TOOL_NAME: &str = "sebi";

/// JSON schema version of SEBI reports.
/// This must be bumped only when SCHEMA.md changes semantically.
pub const SCHEMA_VERSION: &str = "0.1.0";

pub const RULE_CATALOG_VERSION: &str = "0.1.0";
