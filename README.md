# SEBI -- Stylus Execution Boundary Inspector

SEBI is a deterministic, static analysis engine for [Stylus](https://docs.arbitrum.io/stylus/gentle-introduction)-compiled WASM smart contracts.

It inspects WebAssembly binaries *before deployment* to detect execution-boundary risks such as unbounded memory growth, dynamic dispatch, and potentially unbounded control flow -- without executing the contract or relying on chain state.

SEBI produces stable, explainable JSON reports that help developers, auditors, and infrastructure providers reason about execution safety before deployment.

## What SEBI detects

SEBI scans WASM binaries for structural patterns that reduce static predictability:

| Rule | Signal | Severity | What it detects |
|------|--------|----------|-----------------|
| R-MEM-01 | `signals.memory.has_max` | MED | Memory declared without an upper bound |
| R-MEM-02 | `signals.instructions.has_memory_grow` | HIGH | Runtime `memory.grow` instruction present |
| R-CALL-01 | `signals.instructions.has_call_indirect` | HIGH | Dynamic dispatch via `call_indirect` |
| R-LOOP-01 | `signals.instructions.has_loop` | MED | Loop constructs that complicate termination analysis |
| R-SIZE-01 | `artifact.size_bytes` | MED | Artifact exceeds 200 KB size threshold |

Rules are documented in detail in [`docs/RULES.md`](docs/RULES.md).

## Classification

Triggered rules combine into a final risk verdict:

| Level | Exit code | Condition |
|-------|-----------|-----------|
| `SAFE` | 0 | No rules triggered |
| `RISK` | 1 | Any MED severity rule triggered |
| `HIGH_RISK` | 2 | Any HIGH severity rule triggered |

## Prerequisites

- **Rust** toolchain (edition 2024). Tested with rustc 1.85+.
- **Cargo** (comes with Rust).

No additional system dependencies are required.

## Project structure

```
sebi/
  Cargo.toml                    # Workspace root
  README.md
  docs/
    RULES.md                    # Rule catalog specification
    SCHEMA.md                   # Report schema specification
  crates/
    sebi-core/                  # Core analysis library
      src/
        lib.rs                  # Entry point and pipeline orchestration
        wasm/
          read.rs               # Artifact loading and SHA-256 hashing
          parse.rs              # WASM binary parsing orchestration
          sections.rs           # Section-level extraction (memory, imports, exports)
          scan.rs               # Instruction scanning (memory.grow, call_indirect, loop)
          stylus.rs             # Stylus-specific post-parse normalization
        signals/
          model.rs              # Schema-stable signal data structures
          extract.rs            # Mapping from raw facts to signals
        rules/
          catalog.rs            # Rule definitions (IDs, severities, metadata)
          eval.rs               # Rule evaluation engine
          classify.rs           # Risk classification and exit code logic
        report/
          model.rs              # Report data structures (JSON contract)
          render.rs             # Human-readable text output
        util/
          deterministic.rs      # Deterministic sorting utilities
      tests/
        integration.rs          # End-to-end integration tests
        fixtures/               # WAT source files for test contracts
    sebi-cli/                   # CLI frontend (in development)
      src/
        lib.rs
```

## Build

```sh
cargo build
```

## Usage

SEBI's core library exposes a single entry point:

```rust
use sebi_core::report::model::ToolInfo;
use std::path::Path;

let tool = ToolInfo {
    name: "sebi".into(),
    version: "0.1.0".into(),
    commit: None,
};

let report = sebi_core::inspect(Path::new("contract.wasm"), tool)?;

// JSON output
let json = serde_json::to_string_pretty(&report)?;
println!("{json}");

// CI exit code
std::process::exit(report.classification.exit_code);
```

The `inspect` function runs the full pipeline:

1. **Load** -- read the file and compute a SHA-256 hash.
2. **Parse** -- extract WASM sections and scan instructions.
3. **Extract** -- map raw facts to schema-stable signals.
4. **Evaluate** -- check signals against the rule catalog.
5. **Classify** -- derive a risk level and CI exit code.
6. **Report** -- assemble the final JSON report.

## Report format

Reports conform to the schema defined in [`docs/SCHEMA.md`](docs/SCHEMA.md). A report contains:

```json
{
  "schema_version": "0.1.0",
  "tool": { "name": "sebi", "version": "0.1.0" },
  "artifact": { "path": "contract.wasm", "size_bytes": 1234, "hash": { "algorithm": "sha256", "value": "..." } },
  "signals": {
    "module": { "function_count": 10 },
    "memory": { "memory_count": 1, "min_pages": 2, "max_pages": 16, "has_max": true },
    "imports_exports": { "import_count": 4, "export_count": 3 },
    "instructions": { "has_memory_grow": false, "has_call_indirect": false, "has_loop": true, "loop_count": 2 }
  },
  "analysis": { "status": "ok", "warnings": [] },
  "rules": { "catalog": { "catalog_version": "0.1.0", "ruleset": "default" }, "triggered": [] },
  "classification": { "level": "SAFE", "policy": "default", "exit_code": 0 }
}
```

Key properties:
- **Deterministic**: identical artifacts always produce identical JSON output.
- **No timestamps**: reports contain no nondeterministic values.
- **Sorted arrays**: imports, exports, triggered rules, and rule IDs are sorted deterministically.

## Testing

Run all tests (unit and integration):

```sh
cargo test --package sebi-core
```

Run only integration tests:

```sh
cargo test --package sebi-core --test integration
```

### Integration test fixtures

Integration tests use WAT (WebAssembly Text) fixture files in `crates/sebi-core/tests/fixtures/` that model realistic Stylus contracts compiled from Rust and C++:

| Fixture | Source language | Profile | Rules triggered |
|---------|---------------|---------|-----------------|
| `rust_safe_storage.wat` | Rust | Safe | None |
| `rust_loop_unbounded_mem.wat` | Rust | Risky | R-MEM-01, R-LOOP-01 |
| `rust_dynamic_dispatch.wat` | Rust | High risk | R-MEM-02, R-CALL-01 |
| `cpp_vtable_erc20.wat` | C++ | High risk | R-MEM-01, R-MEM-02, R-CALL-01, R-LOOP-01 |
| `cpp_allocator.wat` | C++ | High risk | R-MEM-02, R-LOOP-01 |
| `minimal_module.wat` | -- | Edge case | R-MEM-01 |
| `imported_memory_bounded.wat` | -- | Edge case | None |
| `imported_memory_unbounded.wat` | -- | Edge case | R-MEM-01 |
| `all_signals.wat` | -- | Edge case | R-MEM-01, R-MEM-02, R-CALL-01, R-LOOP-01 |
| `nested_loops.wat` | -- | Edge case | R-MEM-01, R-LOOP-01 |
| `multiple_memory_grow.wat` | -- | Edge case | R-MEM-01, R-MEM-02 |

Fixtures are compiled from WAT to WASM at test time using the [`wat`](https://crates.io/crates/wat) crate.

## Design principles

- **Separation of concerns**: parsing, signal extraction, rule evaluation, and classification are strictly isolated. Rules never parse WASM directly; they operate only on extracted signals.
- **Determinism**: identical input artifacts produce identical output. No timestamps, no nondeterministic ordering, no random elements.
- **Conservative analysis**: SEBI favors false positives over silent misses. It flags structural capabilities, not confirmed intent.
- **Explainability**: every triggered rule includes structured evidence referencing specific schema paths.

## Non-goals

SEBI does **not** attempt to:

- Execute or simulate WASM
- Estimate gas or runtime cost
- Detect exploits or prove correctness
- Infer developer intent
- Replace audits or runtime enforcement

SEBI reports **structural execution-boundary signals only**.

## Documentation

- [`docs/RULES.md`](docs/RULES.md) -- rule catalog: trigger conditions, severities, evidence, classification policy.
- [`docs/SCHEMA.md`](docs/SCHEMA.md) -- report schema: field specifications, types, determinism guarantees.
