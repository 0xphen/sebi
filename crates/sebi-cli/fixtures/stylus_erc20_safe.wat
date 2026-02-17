(module
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "vm_hooks" "msg_sender" (func $msg_sender (param i32)))
  (import "vm_hooks" "emit_log" (func $emit_log (param i32 i32 i32)))

  (memory (export "memory") 2 16)

  (func $total_supply
    (call $storage_load (i32.const 0) (i32.const 256))
    (call $write_result (i32.const 256) (i32.const 32))
  )

  (func $balance_of (param $addr_offset i32)
    (call $storage_load (local.get $addr_offset) (i32.const 256))
    (call $write_result (i32.const 256) (i32.const 32))
  )

  (func $transfer (param $to_offset i32) (param $amount_offset i32) (result i32)
    (call $msg_sender (i32.const 512))
    (call $storage_load (i32.const 512) (i32.const 576))
    (call $storage_load (local.get $to_offset) (i32.const 640))

    (if (i32.lt_u
          (i32.load (i32.const 576))
          (i32.load (local.get $amount_offset)))
      (then (return (i32.const 1)))
    )

    (i32.store (i32.const 576)
      (i32.sub (i32.load (i32.const 576)) (i32.load (local.get $amount_offset)))
    )
    (call $storage_store (i32.const 512) (i32.const 576))

    (i32.store (i32.const 640)
      (i32.add (i32.load (i32.const 640)) (i32.load (local.get $amount_offset)))
    )
    (call $storage_store (local.get $to_offset) (i32.const 640))

    (call $emit_log (i32.const 512) (i32.const 32) (i32.const 1))

    (i32.const 0)
  )

  (func $approve (param $spender_offset i32) (param $amount_offset i32)
    (call $msg_sender (i32.const 512))
    (call $storage_store (local.get $spender_offset) (local.get $amount_offset))
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (call $read_args (i32.const 0))
    (local.set $selector (i32.load (i32.const 0)))

    (if (i32.eqz (local.get $selector))
      (then (call $total_supply))
      (else
        (if (i32.eq (local.get $selector) (i32.const 1))
          (then (call $balance_of (i32.const 4)))
          (else
            (if (i32.eq (local.get $selector) (i32.const 2))
              (then (drop (call $transfer (i32.const 4) (i32.const 36))))
              (else
                (if (i32.eq (local.get $selector) (i32.const 3))
                  (then (call $approve (i32.const 4) (i32.const 36)))
                )
              )
            )
          )
        )
      )
    )
    (i32.const 0)
  )

  (func (export "mark_used"))

  (data (i32.const 768) "StylusToken")
  (data (i32.const 800) "STK")
)
