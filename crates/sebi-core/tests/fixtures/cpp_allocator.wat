(module
  (import "env" "read_args" (func $read_args (param i32)))
  (import "env" "write_result" (func $write_result (param i32 i32)))
  (import "env" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "env" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  (memory (export "memory") 4 64)

  (func $operator_new (param $size i32) (result i32)
    (local $current_pages i32)
    (local $needed_pages i32)
    (local.set $needed_pages
      (i32.div_u
        (i32.add (local.get $size) (i32.const 65535))
        (i32.const 65536)
      )
    )
    (local.set $current_pages (memory.grow (local.get $needed_pages)))
    (if (i32.eq (local.get $current_pages) (i32.const -1))
      (then (unreachable))
    )
    (i32.mul (local.get $current_pages) (i32.const 65536))
  )

  (func $memcpy (param $dst i32) (param $src i32) (param $len i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $end
      (loop $byte
        (br_if $end (i32.ge_u (local.get $i) (local.get $len)))
        (i32.store8
          (i32.add (local.get $dst) (local.get $i))
          (i32.load8_u (i32.add (local.get $src) (local.get $i)))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $byte)
      )
    )
  )

  (func $memset (param $ptr i32) (param $val i32) (param $len i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $end
      (loop $fill
        (br_if $end (i32.ge_u (local.get $i) (local.get $len)))
        (i32.store8
          (i32.add (local.get $ptr) (local.get $i))
          (local.get $val)
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $fill)
      )
    )
  )

  (func $store_string (param $data_ptr i32) (param $data_len i32) (param $slot i32)
    (local $buf i32)
    (local.set $buf (call $operator_new (local.get $data_len)))
    (call $memcpy (local.get $buf) (local.get $data_ptr) (local.get $data_len))
    (call $storage_store (local.get $buf) (local.get $slot))
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (call $read_args (i32.const 0))
    (call $store_string (i32.const 4) (i32.sub (local.get $args_len) (i32.const 4)) (i32.const 0))
    (i32.const 0)
  )

  (func (export "mark_used"))
)
