(module
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  (memory (export "memory") 1 4)

  (data (i32.const 0) "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00")

  (func $get_counter
    (call $storage_load (i32.const 0) (i32.const 64))
    (call $write_result (i32.const 64) (i32.const 32))
  )

  (func $increment
    (call $storage_load (i32.const 0) (i32.const 64))
    (i32.store
      (i32.const 64)
      (i32.add (i32.load (i32.const 64)) (i32.const 1))
    )
    (call $storage_store (i32.const 0) (i32.const 64))
  )

  (func $decrement
    (call $storage_load (i32.const 0) (i32.const 64))
    (i32.store
      (i32.const 64)
      (i32.sub (i32.load (i32.const 64)) (i32.const 1))
    )
    (call $storage_store (i32.const 0) (i32.const 64))
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (call $read_args (i32.const 128))
    (local.set $selector (i32.load8_u (i32.const 128)))

    (if (i32.eqz (local.get $selector))
      (then (call $get_counter))
      (else
        (if (i32.eq (local.get $selector) (i32.const 1))
          (then (call $increment))
          (else (call $decrement))
        )
      )
    )
    (i32.const 0)
  )

  (func (export "mark_used"))
)
