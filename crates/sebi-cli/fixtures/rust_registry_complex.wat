(module
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "vm_hooks" "msg_sender" (func $msg_sender (param i32)))
  (import "vm_hooks" "block_number" (func $block_number (result i64)))

  (memory (export "memory") 4)

  (type $register_t (func (param i32 i32) (result i32)))
  (type $lookup_t (func (param i32) (result i32)))

  (table 6 funcref)
  (elem (i32.const 0)
    $register_name
    $register_address
    $lookup_by_name 
    $lookup_by_addr
    $list_all 
    $admin_clear 
  )

  (global $heap_ptr (mut i32) (i32.const 8192))

  (func $alloc (param $size i32) (result i32)
    (local $ptr i32)
    (local $needed i32)
    (local.set $ptr (global.get $heap_ptr))

    (if (i32.gt_u
          (i32.add (global.get $heap_ptr) (local.get $size))
          (i32.mul (memory.size) (i32.const 65536)))
      (then
        (local.set $needed
          (i32.div_u
            (i32.add (local.get $size) (i32.const 65535))
            (i32.const 65536)
          )
        )
        (if (i32.eq (memory.grow (local.get $needed)) (i32.const -1))
          (then (unreachable))
        )
      )
    )

    (global.set $heap_ptr (i32.add (global.get $heap_ptr) (local.get $size)))
    (local.get $ptr)
  )

  (func $register_name (param $data_ptr i32) (param $data_len i32) (result i32)
    (call $msg_sender (i32.const 256))
    (call $storage_store (local.get $data_ptr) (i32.const 256))
    (i32.const 0)
  )

  (func $register_address (param $data_ptr i32) (param $data_len i32) (result i32)
    (call $msg_sender (i32.const 256))
    (call $storage_store (i32.const 256) (local.get $data_ptr))
    (i32.const 0)
  )

  (func $lookup_by_name (param $key_ptr i32) (result i32)
    (call $storage_load (local.get $key_ptr) (i32.const 512))
    (call $write_result (i32.const 512) (i32.const 32))
    (i32.const 0)
  )

  (func $lookup_by_addr (param $addr_ptr i32) (result i32)
    (call $storage_load (local.get $addr_ptr) (i32.const 512))
    (call $write_result (i32.const 512) (i32.const 32))
    (i32.const 0)
  )

  (func $list_all (param $count_ptr i32) (result i32)
    (local $count i32)
    (local $i i32)
    (local $buf i32)

    (local.set $count (i32.load (local.get $count_ptr)))
    (local.set $buf (call $alloc (i32.mul (local.get $count) (i32.const 32))))
    (local.set $i (i32.const 0))

    (block $done
      (loop $iter
        (br_if $done (i32.ge_u (local.get $i) (local.get $count)))
        (call $storage_load
          (i32.add (i32.const 1024) (i32.mul (local.get $i) (i32.const 32)))
          (i32.add (local.get $buf) (i32.mul (local.get $i) (i32.const 32)))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $iter)
      )
    )

    (call $write_result (local.get $buf) (i32.mul (local.get $count) (i32.const 32)))
    (i32.const 0)
  )

  (func $admin_clear (param $slot_ptr i32) (result i32)
    (call $msg_sender (i32.const 256))
    (i64.store (i32.const 320) (i64.const 0))
    (i64.store (i32.const 328) (i64.const 0))
    (i64.store (i32.const 336) (i64.const 0))
    (i64.store (i32.const 344) (i64.const 0))
    (call $storage_store (local.get $slot_ptr) (i32.const 320))
    (i32.const 0)
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (local $payload_ptr i32)

    (call $read_args (i32.const 0))
    (local.set $selector (i32.load (i32.const 0)))

    (local.set $payload_ptr (call $alloc (i32.sub (local.get $args_len) (i32.const 4))))

    (if (i32.lt_u (local.get $selector) (i32.const 4))
      (then
        (drop (call_indirect (type $register_t)
          (local.get $payload_ptr)
          (i32.sub (local.get $args_len) (i32.const 4))
          (local.get $selector)
        ))
      )
      (else
        (if (i32.lt_u (local.get $selector) (i32.const 6))
          (then
            (drop (call_indirect (type $lookup_t)
              (local.get $payload_ptr)
              (local.get $selector)
            ))
          )
        )
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))
)
