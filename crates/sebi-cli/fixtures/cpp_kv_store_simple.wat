(module
  (import "env" "read_args" (func $read_args (param i32)))
  (import "env" "write_result" (func $write_result (param i32 i32)))
  (import "env" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "env" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  (memory (export "memory") 2 8)

  (func $memcpy (param $dst i32) (param $src i32) (param $len i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $end
      (loop $copy
        (br_if $end (i32.ge_u (local.get $i) (local.get $len)))
        (i32.store8
          (i32.add (local.get $dst) (local.get $i))
          (i32.load8_u (i32.add (local.get $src) (local.get $i)))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $copy)
      )
    )
  )

  (func $kv_put (param $key_offset i32) (param $val_offset i32)
    (call $memcpy (i32.const 256) (local.get $key_offset) (i32.const 32))
    (call $memcpy (i32.const 320) (local.get $val_offset) (i32.const 32))
    (call $storage_store (i32.const 256) (i32.const 320))
  )

  (func $kv_get (param $key_offset i32)
    (call $memcpy (i32.const 256) (local.get $key_offset) (i32.const 32))
    (call $storage_load (i32.const 256) (i32.const 320))
    (call $write_result (i32.const 320) (i32.const 32))
  )

  (func $kv_del (param $key_offset i32)
    (call $memcpy (i32.const 256) (local.get $key_offset) (i32.const 32))
    (i64.store (i32.const 320) (i64.const 0))
    (i64.store (i32.const 328) (i64.const 0))
    (i64.store (i32.const 336) (i64.const 0))
    (i64.store (i32.const 344) (i64.const 0))
    (call $storage_store (i32.const 256) (i32.const 320))
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $op i32)
    (call $read_args (i32.const 0))
    (local.set $op (i32.load8_u (i32.const 0)))

    (if (i32.eqz (local.get $op))
      (then (call $kv_put (i32.const 1) (i32.const 33)))
      (else
        (if (i32.eq (local.get $op) (i32.const 1))
          (then (call $kv_get (i32.const 1)))
          (else (call $kv_del (i32.const 1)))
        )
      )
    )
    (i32.const 0)
  )

  (func (export "mark_used"))
)
