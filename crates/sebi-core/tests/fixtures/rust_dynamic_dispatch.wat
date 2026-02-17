(module
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  (memory (export "memory") 2 256)

  (type $handler_t (func (param i32) (result i32)))

  (table 4 funcref)
  (elem (i32.const 0) $handle_transfer $handle_approve $handle_balance $handle_metadata)

  (func $handle_transfer (param $arg i32) (result i32)
    (call $storage_load (i32.const 64) (i32.const 128))
    (call $storage_store (i32.const 64) (i32.const 128))
    (i32.const 0)
  )

  (func $handle_approve (param $arg i32) (result i32)
    (call $storage_store (i32.const 64) (i32.const 128))
    (i32.const 0)
  )

  (func $handle_balance (param $arg i32) (result i32)
    (call $storage_load (i32.const 64) (i32.const 128))
    (call $write_result (i32.const 128) (i32.const 32))
    (i32.const 0)
  )

  (func $handle_metadata (param $arg i32) (result i32)
    (call $write_result (i32.const 256) (i32.const 64))
    (i32.const 0)
  )

  (func $alloc (param $pages i32) (result i32)
    (memory.grow (local.get $pages))
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (call $read_args (i32.const 0))

    (drop (call $alloc (i32.const 1)))

    (local.set $selector (i32.load (i32.const 0)))

    (if (i32.lt_u (local.get $selector) (i32.const 4))
      (then
        (drop (call_indirect (type $handler_t) (local.get $selector) (local.get $selector)))
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))
)
