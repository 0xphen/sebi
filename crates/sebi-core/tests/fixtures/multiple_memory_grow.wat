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
