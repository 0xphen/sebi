(module
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "vm_hooks" "msg_sender" (func $msg_sender (param i32)))
  (import "vm_hooks" "msg_value" (func $msg_value (param i32)))
  (import "vm_hooks" "emit_log" (func $emit_log (param i32 i32 i32)))
  (import "vm_hooks" "block_number" (func $block_number (result i64)))

  (memory (export "memory") 4)

  (type $pool_adapter_t (func (param i32 i32 i32 i32) (result i32)))

  (type $callback_t (func (param i32) (result i32)))

  (table 8 funcref)
  (elem (i32.const 0)
    $adapter_uniswap_v2 
    $adapter_uniswap_v3
    $adapter_curve 
    $adapter_balancer
    $quote_uniswap_v2
    $quote_uniswap_v3 
    $quote_curve
    $quote_balancer 
  )

  (global $heap_ptr (mut i32) (i32.const 16384))

  (func $alloc (param $size i32) (result i32)
    (local $ptr i32)
    (local $pages i32)
    (local.set $ptr (global.get $heap_ptr))

    (if (i32.gt_u
          (i32.add (global.get $heap_ptr) (local.get $size))
          (i32.mul (memory.size) (i32.const 65536)))
      (then
        (local.set $pages
          (i32.add
            (i32.div_u (local.get $size) (i32.const 65536))
            (i32.const 1)
          )
        )
        (if (i32.eq (memory.grow (local.get $pages)) (i32.const -1))
          (then (unreachable))
        )
      )
    )

    (global.set $heap_ptr (i32.add (global.get $heap_ptr) (local.get $size)))
    (local.get $ptr)
  )

  (func $memcpy (param $dst i32) (param $src i32) (param $len i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $end
      (loop $cp
        (br_if $end (i32.ge_u (local.get $i) (local.get $len)))
        (i32.store8
          (i32.add (local.get $dst) (local.get $i))
          (i32.load8_u (i32.add (local.get $src) (local.get $i)))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $cp)
      )
    )
  )

  (func $adapter_uniswap_v2 (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (call $storage_load (local.get $out_ptr) (i32.const 1088))
    (i32.store (i32.const 1152)
      (i32.div_u
        (i32.mul (i32.load (i32.const 1088)) (local.get $amount))
        (i32.add (i32.load (i32.const 1024)) (local.get $amount))
      )
    )
    (call $storage_store (local.get $in_ptr) (i32.const 1024))
    (call $storage_store (local.get $out_ptr) (i32.const 1088))
    (i32.load (i32.const 1152))
  )

  (func $adapter_uniswap_v3 (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (i32.store (i32.const 1152)
      (i32.div_u (i32.mul (i32.load (i32.const 1024)) (local.get $amount)) (i32.const 1000))
    )
    (call $storage_store (local.get $in_ptr) (i32.const 1024))
    (i32.load (i32.const 1152))
  )

  (func $adapter_curve (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (i32.store (i32.const 1152)
      (i32.sub (local.get $amount) (i32.div_u (local.get $amount) (i32.const 2500)))
    )
    (call $storage_store (local.get $in_ptr) (i32.const 1024))
    (i32.load (i32.const 1152))
  )

  (func $adapter_balancer (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (call $storage_load (local.get $out_ptr) (i32.const 1088))
    (i32.store (i32.const 1152)
      (i32.div_u
        (i32.mul (i32.load (i32.const 1088)) (local.get $amount))
        (i32.add (i32.load (i32.const 1024)) (local.get $amount))
      )
    )
    (call $storage_store (local.get $in_ptr) (i32.const 1024))
    (i32.load (i32.const 1152))
  )

  (func $quote_uniswap_v2 (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (call $storage_load (local.get $out_ptr) (i32.const 1088))
    (i32.div_u
      (i32.mul (i32.load (i32.const 1088)) (local.get $amount))
      (i32.add (i32.load (i32.const 1024)) (local.get $amount))
    )
  )

  (func $quote_uniswap_v3 (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (i32.div_u (i32.mul (i32.load (i32.const 1024)) (local.get $amount)) (i32.const 1000))
  )

  (func $quote_curve (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (i32.sub (local.get $amount) (i32.div_u (local.get $amount) (i32.const 2500)))
  )

  (func $quote_balancer (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (call $storage_load (local.get $out_ptr) (i32.const 1088))
    (i32.div_u
      (i32.mul (i32.load (i32.const 1088)) (local.get $amount))
      (i32.add (i32.load (i32.const 1024)) (local.get $amount))
    )
  )

  (func $execute_swap_path (param $path_ptr i32) (param $hop_count i32) (param $initial_amount i32) (result i32)
    (local $i i32)
    (local $amount i32)
    (local $offset i32)
    (local $pool_type i32)
    (local $pool_id i32)
    (local $token_in i32)
    (local $token_out i32)

    (local.set $amount (local.get $initial_amount))
    (local.set $i (i32.const 0))

    (block $done
      (loop $hop
        (br_if $done (i32.ge_u (local.get $i) (local.get $hop_count)))

        (local.set $offset
          (i32.add (local.get $path_ptr) (i32.mul (local.get $i) (i32.const 69)))
        )
        (local.set $pool_type (i32.load8_u (local.get $offset)))
        (local.set $pool_id (i32.load (i32.add (local.get $offset) (i32.const 1))))
        (local.set $token_in (i32.add (local.get $offset) (i32.const 5)))
        (local.set $token_out (i32.add (local.get $offset) (i32.const 37)))

        (if (i32.lt_u (local.get $pool_type) (i32.const 4))
          (then
            (local.set $amount
              (call_indirect (type $pool_adapter_t)
                (local.get $pool_id)
                (local.get $token_in)
                (local.get $token_out)
                (local.get $amount)
                (local.get $pool_type)
              )
            )
          )
        )

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $hop)
      )
    )

    (local.get $amount)
  )

  (func $quote_swap_path (param $path_ptr i32) (param $hop_count i32) (param $initial_amount i32) (result i32)
    (local $i i32)
    (local $amount i32)
    (local $offset i32)
    (local $pool_type i32)

    (local.set $amount (local.get $initial_amount))
    (local.set $i (i32.const 0))

    (block $done
      (loop $hop
        (br_if $done (i32.ge_u (local.get $i) (local.get $hop_count)))

        (local.set $offset
          (i32.add (local.get $path_ptr) (i32.mul (local.get $i) (i32.const 69)))
        )
        (local.set $pool_type (i32.load8_u (local.get $offset)))

        (if (i32.lt_u (local.get $pool_type) (i32.const 4))
          (then
            (local.set $amount
              (call_indirect (type $pool_adapter_t)
                (i32.load (i32.add (local.get $offset) (i32.const 1)))
                (i32.add (local.get $offset) (i32.const 5))
                (i32.add (local.get $offset) (i32.const 37))
                (local.get $amount)
                (i32.add (local.get $pool_type) (i32.const 4))
              )
            )
          )
        )

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $hop)
      )
    )

    (local.get $amount)
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (local $hop_count i32)
    (local $amount i32)
    (local $path_buf i32)
    (local $path_len i32)
    (local $result i32)

    (call $read_args (i32.const 0))
    (local.set $selector (i32.load (i32.const 0)))

    (if (i32.le_u (local.get $selector) (i32.const 1))
      (then
        (local.set $hop_count (i32.load (i32.const 4)))
        (local.set $amount (i32.load (i32.const 8)))
        (local.set $path_len (i32.mul (local.get $hop_count) (i32.const 69)))

        (local.set $path_buf (call $alloc (local.get $path_len)))
        (call $memcpy (local.get $path_buf) (i32.const 12) (local.get $path_len))

        (if (i32.eqz (local.get $selector))
          (then
            (local.set $result
              (call $execute_swap_path
                (local.get $path_buf)
                (local.get $hop_count)
                (local.get $amount)
              )
            )
            (i32.store (i32.const 2048) (local.get $result))
            (call $emit_log (i32.const 2048) (i32.const 4) (i32.const 2))
          )
          (else
            (local.set $result
              (call $quote_swap_path
                (local.get $path_buf)
                (local.get $hop_count)
                (local.get $amount)
              )
            )
          )
        )

        (i32.store (i32.const 2048) (local.get $result))
        (call $write_result (i32.const 2048) (i32.const 4))
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))
)
