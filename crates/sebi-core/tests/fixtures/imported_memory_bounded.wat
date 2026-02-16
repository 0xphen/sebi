;; Stylus contract with bounded imported memory.
;; Models the common Stylus pattern where the host provides bounded memory.
;; No dangerous instructions. R-MEM-01 does NOT fire (has_max is true).
;; Expected classification: SAFE (exit code 0)
(module
  ;; Host provides bounded memory (min 1, max 16 pages)
  (import "env" "memory" (memory 1 16))

  ;; Simple function that does basic arithmetic -- no loops, no grow, no indirect
  (func $add (export "add") (param $a i32) (param $b i32) (result i32)
    (i32.add (local.get $a) (local.get $b))
  )

  (func (export "mark_used"))
)
