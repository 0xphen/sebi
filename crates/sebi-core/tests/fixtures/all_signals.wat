;; Contract that triggers ALL instruction-level rules simultaneously.
;; Triggers: R-MEM-01 (no max), R-MEM-02 (memory.grow), R-CALL-01 (call_indirect), R-LOOP-01 (loop).
;; Expected classification: HIGH_RISK (exit code 2)
(module
  ;; Unbounded memory (no max) -- triggers R-MEM-01
  (memory (export "memory") 1)

  ;; Function table for indirect calls
  (type $ft (func (result i32)))
  (table 2 funcref)
  (elem (i32.const 0) $worker_a $worker_b)

  (func $worker_a (result i32) (i32.const 42))
  (func $worker_b (result i32) (i32.const 99))

  ;; Function combining all risky patterns
  (func $everything (export "everything") (result i32)
    (local $i i32)
    (local $sum i32)

    ;; memory.grow -- triggers R-MEM-02
    (drop (memory.grow (i32.const 1)))

    ;; loop -- triggers R-LOOP-01
    (local.set $i (i32.const 0))
    (block $exit
      (loop $repeat
        (br_if $exit (i32.ge_u (local.get $i) (i32.const 2)))
        ;; call_indirect -- triggers R-CALL-01
        (local.set $sum
          (i32.add
            (local.get $sum)
            (call_indirect (type $ft) (local.get $i))
          )
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $repeat)
      )
    )

    (local.get $sum)
  )
)
