;; Contract with multiple memory.grow calls across multiple functions.
;; Tests accurate counting of memory.grow occurrences.
;; Triggers: R-MEM-02 (3x memory.grow), R-MEM-01 (no max).
;; Expected: memory_grow_count == 3
(module
  (memory (export "memory") 1)

  (func $grow_once (drop (memory.grow (i32.const 1))))

  (func $grow_twice
    (drop (memory.grow (i32.const 2)))
    (drop (memory.grow (i32.const 1)))
  )

  (func $main (export "main")
    (call $grow_once)
    (call $grow_twice)
  )
)
