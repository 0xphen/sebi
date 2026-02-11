use serde::{Deserialize, Serialize};

/// Raw observations extracted from a WASM artifact.
/// Maps to the `signals` object in the SEBI report schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signals {
    pub module: ModuleSignals,
    pub memory: MemorySignals,
    pub imports_exports: ImportExportSignals,
    pub instructions: InstructionSignals,
}

/// Structural facts derived from WASM sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSignals {
    /// Count of defined functions; excludes imports.
    pub function_count: u32,
    pub section_count: Option<u32>,
}

/// Declared memory boundaries and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySignals {
    pub memory_count: u32,
    /// Size in 64 KiB pages.
    pub min_pages: Option<u64>,
    /// Size in 64 KiB pages. `None` indicates no upper bound.
    pub max_pages: Option<u64>,
    pub has_max: bool,
}

/// Summary of external interfaces.
/// Lists are sorted deterministically if present.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportExportSignals {
    pub import_count: u32,
    pub export_count: u32,
    pub imports: Option<Vec<ImportItem>>,
    pub exports: Option<Vec<ExportItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportItem {
    pub module: String,
    pub name: String,
    /// External kind: e.g., "func", "memory", "table", "global", "tag".
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportItem {
    pub name: String,
    /// External kind: e.g., "func", "memory", "table", "global", "tag".
    pub kind: String,
}

/// Capability indicators detected during function body scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionSignals {
    pub has_memory_grow: bool,
    pub memory_grow_count: u64,
    pub has_call_indirect: bool,
    pub call_indirect_count: u64,
    pub has_loop: bool,
    pub loop_count: u64,
}
