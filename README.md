# SEBI -- Stylus Execution Boundary Inspector

SEBI is a deterministic, static analysis engine for [Stylus](https://docs.arbitrum.io/stylus/gentle-introduction)-compiled WASM smart contracts. It inspects WebAssembly binaries **before deployment** to detect execution-boundary risks -- without executing the contract or relying on chain state.

SEBI produces stable, explainable JSON reports that help developers, auditors, and infrastructure providers reason about execution safety before a contract goes on-chain.

## Table of Contents

- [What SEBI Detects](#what-sebi-detects)
- [Classification](#classification)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
  - [CLI](#cli)
  - [Library](#library)
- [Report Format](#report-format)
- [Project Structure](#project-structure)
- [Testing](#testing)
- [Design Principles](#design-principles)
- [Non-Goals](#non-goals)
- [Documentation](#documentation)
- [License](#license)

## What SEBI Detects

SEBI scans WASM binaries for structural patterns that reduce static predictability:

| Rule | Signal | Severity | What it detects |
|------|--------|----------|-----------------|
| R-MEM-01 | `signals.memory.has_max` | MED | Memory declared without an upper bound |
| R-MEM-02 | `signals.instructions.has_memory_grow` | HIGH | Runtime `memory.grow` instruction present |
| R-CALL-01 | `signals.instructions.has_call_indirect` | HIGH | Dynamic dispatch via `call_indirect` |
| R-LOOP-01 | `signals.instructions.has_loop` | MED | Loop constructs that complicate termination analysis |
| R-SIZE-01 | `artifact.size_bytes` | MED | Artifact exceeds 200 KB size threshold |

See [`docs/RULES.md`](docs/RULES.md) for detailed trigger conditions, evidence, and rationale.

## Classification

Triggered rules combine into a final risk verdict:

| Level | Exit Code | Condition |
|-------|-----------|-----------|
| `SAFE` | 0 | No MED or HIGH severity rules triggered |
| `RISK` | 1 | At least one MED severity rule triggered (no HIGH) |
| `HIGH_RISK` | 2 | At least one HIGH severity rule triggered |

The exit code makes SEBI directly usable as a CI gate -- a non-zero exit signals risk.

## Getting Started

### Prerequisites

- **Rust** toolchain (edition 2024), rustc **1.85** or later
- **Cargo** (included with Rust)

No additional system dependencies are required.

Install Rust via [rustup](https://rustup.rs/) if you don't have it:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installation

**Build from source:**

```sh
git clone https://github.com/aspect-build/sebi.git
cd sebi
cargo build --release
```

The compiled binary is at `target/release/sebi-cli`.

**Install directly with Cargo:**

```sh
cargo install --path crates/sebi-cli
```

This places `sebi-cli` in your Cargo bin directory (typically `~/.cargo/bin/`).

## Usage

### CLI

The `sebi-cli` binary inspects a WASM file and outputs a structured report.

```sh
sebi-cli <WASM_FILE> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<WASM_FILE>` | Path to the `.wasm` artifact to inspect |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--format <FORMAT>` | `json` | Output format: `json` or `text` |
| `--out <FILE>` | stdout | Write output to a file instead of stdout |
| `--commit <HASH>` | none | Git commit hash to embed in report metadata |
| `-h, --help` | | Print help information |
| `-V, --version` | | Print version |

**Examples:**

Inspect a contract and print the JSON report to stdout:

```sh
sebi-cli contract.wasm
```

Inspect with human-readable text output:

```sh
sebi-cli contract.wasm --format text
```

Save a JSON report to a file:

```sh
sebi-cli contract.wasm --out report.json
```

Embed a commit hash in the report metadata:

```sh
sebi-cli contract.wasm --commit $(git rev-parse HEAD)
```

Use as a CI gate (non-zero exit on risk):

```sh
sebi-cli contract.wasm || echo "Risk detected"
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| `0` | `SAFE` -- no MED or HIGH severity rules triggered |
| `1` | `RISK` -- at least one MED severity rule triggered (no HIGH) |
| `2` | `HIGH_RISK` -- at least one HIGH severity rule triggered |

### Library

SEBI's core library (`sebi-core`) can be used as a Rust dependency:

```toml
[dependencies]
sebi-core = { path = "crates/sebi-core" }
```

```rust
use sebi_core::inspect;
use sebi_core::report::model::ToolInfo;
use std::path::Path;

let tool = ToolInfo {
    name: "my-tool".into(),
    version: "1.0.0".into(),
    commit: None,
};

let report = inspect(Path::new("contract.wasm"), tool)?;

// JSON output
let json = serde_json::to_string_pretty(&report)?;
println!("{json}");

// Use exit code for CI gating
std::process::exit(report.classification.exit_code);
```

The `inspect` function runs the full pipeline:

1. **Load** -- read the file and compute a SHA-256 hash
2. **Parse** -- extract WASM sections and scan instructions
3. **Extract** -- map raw facts to schema-stable signals
4. **Evaluate** -- check signals against the rule catalog
5. **Classify** -- derive a risk level and CI exit code
6. **Report** -- assemble the final JSON report

## Report Format

Reports conform to the schema in [`docs/SCHEMA.md`](docs/SCHEMA.md):

```json
{
  "schema_version": "0.1.0",
  "tool": { "name": "sebi-cli", "version": "0.1.0", "commit": null },
  "artifact": {
    "path": "contract.wasm",
    "size_bytes": 1234,
    "hash": { "algorithm": "sha256", "value": "abcdef..." }
  },
  "signals": {
    "module": { "function_count": 10, "section_count": 8 },
    "memory": { "memory_count": 1, "min_pages": 2, "max_pages": 16, "has_max": true },
    "imports_exports": { "import_count": 4, "export_count": 3 },
    "instructions": {
      "has_memory_grow": false, "memory_grow_count": 0,
      "has_call_indirect": false, "call_indirect_count": 0,
      "has_loop": true, "loop_count": 2
    }
  },
  "analysis": { "status": "ok", "warnings": [] },
  "rules": {
    "catalog": { "catalog_version": "0.1.0", "ruleset": "default" },
    "triggered": []
  },
  "classification": {
    "level": "SAFE",
    "policy": "default",
    "reason": "no rules triggered",
    "highest_severity": "NONE",
    "triggered_rule_ids": [],
    "exit_code": 0
  }
}
```

**Key properties:**

- **Deterministic** -- identical artifacts always produce identical JSON
- **No timestamps** -- reports contain no nondeterministic values
- **Sorted arrays** -- imports, exports, triggered rules, and rule IDs are sorted deterministically

## Project Structure

```
sebi/
├── Cargo.toml                          # Workspace root
├── README.md
├── docs/
│   ├── RULES.md                        # Rule catalog specification
│   └── SCHEMA.md                       # Report schema specification
└── crates/
    ├── sebi-core/                      # Core analysis library
    │   ├── src/
    │   │   ├── lib.rs                  # Entry point and pipeline orchestration
    │   │   ├── wasm/                   # WASM parsing and scanning
    │   │   │   ├── read.rs             # Artifact loading, SHA-256 hashing
    │   │   │   ├── parse.rs            # Binary parsing orchestration
    │   │   │   ├── sections.rs         # Section extraction (memory, imports, exports)
    │   │   │   ├── scan.rs             # Instruction scanning (memory.grow, call_indirect, loop)
    │   │   │   └── stylus.rs           # Stylus-specific normalization
    │   │   ├── signals/                # Signal extraction
    │   │   │   ├── model.rs            # Schema-stable data structures
    │   │   │   └── extract.rs          # Raw facts to signals mapping
    │   │   ├── rules/                  # Rule evaluation and classification
    │   │   │   ├── catalog.rs          # Rule definitions (IDs, severities, metadata)
    │   │   │   ├── eval.rs             # Rule evaluation engine
    │   │   │   └── classify.rs         # Risk classification and exit code logic
    │   │   ├── report/                 # Report assembly and rendering
    │   │   │   ├── model.rs            # Report data structures (JSON contract)
    │   │   │   └── render.rs           # Human-readable text output
    │   │   └── util/
    │   │       └── deterministic.rs    # Deterministic sorting utilities
    │   └── tests/
    │       ├── integration.rs          # End-to-end integration tests
    │       └── fixtures/               # WAT source files for test contracts
    └── sebi-cli/                       # CLI frontend
        ├── src/
        │   ├── main.rs                 # CLI entry point
        │   └── args.rs                 # Argument parsing (clap)
        ├── fixtures/                   # Compiled WASM fixtures for CLI tests
        └── tests/
            └── cli.rs                  # CLI integration tests
```

## Testing

Run all tests across the workspace:

```sh
cargo test --workspace
```

Run only `sebi-core` tests (library + integration):

```sh
cargo test --package sebi-core
```

Run only `sebi-core` integration tests:

```sh
cargo test --package sebi-core --test integration
```

Run only `sebi-cli` integration tests:

```sh
cargo test --package sebi-cli
```

### Test Fixtures

`sebi-core` integration tests use WAT (WebAssembly Text) fixtures compiled to WASM at test time via the [`wat`](https://crates.io/crates/wat) crate:

| Fixture | Profile | Rules Triggered |
|---------|---------|-----------------|
| `rust_safe_storage.wat` | Safe | None |
| `rust_loop_unbounded_mem.wat` | Risky | R-MEM-01, R-LOOP-01 |
| `rust_dynamic_dispatch.wat` | High risk | R-MEM-02, R-CALL-01 |
| `cpp_vtable_erc20.wat` | High risk | R-MEM-01, R-MEM-02, R-CALL-01, R-LOOP-01 |
| `cpp_allocator.wat` | High risk | R-MEM-02, R-LOOP-01 |
| `minimal_module.wat` | Edge case | R-MEM-01 |
| `imported_memory_bounded.wat` | Edge case | None |
| `imported_memory_unbounded.wat` | Edge case | R-MEM-01 |
| `all_signals.wat` | Edge case | R-MEM-01, R-MEM-02, R-CALL-01, R-LOOP-01 |
| `nested_loops.wat` | Edge case | R-MEM-01, R-LOOP-01 |
| `multiple_memory_grow.wat` | Edge case | R-MEM-01, R-MEM-02 |

`sebi-cli` integration tests use pre-compiled WASM fixtures in `crates/sebi-cli/fixtures/` to test the binary end-to-end (exit codes, output formats, flag handling).

## Design Principles

- **Separation of concerns** -- parsing, signal extraction, rule evaluation, and classification are strictly isolated. Rules never parse WASM directly; they operate only on extracted signals.
- **Determinism** -- identical input artifacts produce identical output. No timestamps, no nondeterministic ordering.
- **Conservative analysis** -- SEBI favors false positives over silent misses. It flags structural capabilities, not confirmed intent.
- **Explainability** -- every triggered rule includes structured evidence referencing specific schema paths.

## Non-Goals

SEBI does **not** attempt to:

- Execute or simulate WASM
- Estimate gas or runtime cost
- Detect exploits or prove correctness
- Infer developer intent
- Replace audits or runtime enforcement

SEBI reports **structural execution-boundary signals only**.

## Documentation

- [`docs/RULES.md`](docs/RULES.md) -- rule catalog: trigger conditions, severities, evidence, classification policy
- [`docs/SCHEMA.md`](docs/SCHEMA.md) -- report schema: field specifications, types, determinism guarantees

## License

See [LICENSE](LICENSE) for details.
