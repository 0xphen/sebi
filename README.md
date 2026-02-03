# SEBI — Stylus Execution Boundary Inspector

SEBI is a deterministic, static analysis engine for Stylus-compiled WASM smart contracts.

It inspects WebAssembly binaries *before deployment* to detect execution-boundary risks such as
unbounded memory growth, dynamic dispatch, and potentially unbounded control flow — without
executing the contract or relying on chain state.

SEBI is designed to run both as a standalone CLI and as a CI preflight safety check, producing
stable, explainable reports that help developers, auditors, and infrastructure providers reason
about execution safety before deployment.
