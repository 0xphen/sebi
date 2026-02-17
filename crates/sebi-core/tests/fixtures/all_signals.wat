(module
  (memory (export "memory") 1)

  (type $ft (func (result i32)))
  (table 2 funcref)
  (elem (i32.const 0) $worker_a $worker_b)

  (func $worker_a (result i32) (i32.const 42))
  (func $worker_b (result i32) (i32.const 99))

  (func $everything (export "everything") (result i32)
    (local $i i32)
    (local $sum i32)

    (drop (memory.grow (i32.const 1)))

    (local.set $i (i32.const 0))
    (block $exit
      (loop $repeat
        (br_if $exit (i32.ge_u (local.get $i) (i32.const 2)))
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
