(module
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  (memory (export "memory") 2 16)

  (func $get (param $key_ptr i32)
    (call $storage_load (local.get $key_ptr) (i32.const 128))
    (call $write_result (i32.const 128) (i32.const 32))
  )

  (func $set (param $payload_ptr i32)
    (call $storage_store (local.get $payload_ptr) (i32.add (local.get $payload_ptr) (i32.const 32)))
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (call $read_args (i32.const 0))

    (if (i32.eqz (i32.load8_u (i32.const 0)))
      (then (call $get (i32.const 1)))
      (else (call $set (i32.const 1)))
    )

    (i32.const 0)
  )

  (func (export "mark_used"))
)
