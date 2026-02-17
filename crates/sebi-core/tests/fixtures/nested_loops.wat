(module
  (memory (export "memory") 1)

  (func $nested (export "nested") (param $n i32) (result i32)
    (local $i i32)
    (local $j i32)
    (local $k i32)
    (local $acc i32)

    (local.set $i (i32.const 0))
    (block $exit_i
      (loop $outer                             
        (br_if $exit_i (i32.ge_u (local.get $i) (local.get $n)))
        (local.set $j (i32.const 0))
        (block $exit_j
          (loop $middle                          
            (br_if $exit_j (i32.ge_u (local.get $j) (local.get $n)))
            (local.set $k (i32.const 0))
            (block $exit_k
              (loop $inner                     
                (br_if $exit_k (i32.ge_u (local.get $k) (local.get $n)))
                (local.set $acc (i32.add (local.get $acc) (i32.const 1)))
                (local.set $k (i32.add (local.get $k) (i32.const 1)))
                (br $inner)
              )
            )
            (local.set $j (i32.add (local.get $j) (i32.const 1)))
            (br $middle)
          )
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $outer)
      )
    )

    (local.get $acc)
  )
)
