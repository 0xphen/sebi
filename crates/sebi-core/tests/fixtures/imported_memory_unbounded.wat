;; Stylus contract with unbounded imported memory.
;; The host provides memory but without a declared maximum.
;; Triggers R-MEM-01 (has_max is false).
;; Expected classification: RISK (exit code 1)
(module
  ;; Host provides unbounded memory (no max pages)
  (import "env" "memory" (memory 2))

  ;; Simple passthrough function
  (func $echo (export "echo") (param $offset i32) (param $len i32) (result i32)
    (local.get $offset)
  )

  (func (export "mark_used"))
)
