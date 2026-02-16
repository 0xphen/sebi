;; Minimal valid WASM module: no functions, no memory, no imports/exports.
;; Edge case: triggers R-MEM-01 (no memory -> has_max is false).
;; Expected classification: RISK (exit code 1)
;; Expected warning: "no memory section or imported memory detected"
(module)
