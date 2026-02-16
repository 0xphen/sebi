;; Stylus-compiled contract: DEX aggregation router.
;; Models a complex Stylus contract that routes swaps across multiple pools,
;; uses dynamic dispatch for pool adapters, a bump allocator for intermediate
;; buffers, and iterative path finding with loops.
;; Triggers: R-MEM-01 (no max), R-MEM-02 (memory.grow), R-CALL-01 (call_indirect), R-LOOP-01 (loop).
;; Expected classification: HIGH_RISK (exit code 2)
(module
  ;; Stylus VM host imports
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "vm_hooks" "msg_sender" (func $msg_sender (param i32)))
  (import "vm_hooks" "msg_value" (func $msg_value (param i32)))
  (import "vm_hooks" "emit_log" (func $emit_log (param i32 i32 i32)))
  (import "vm_hooks" "block_number" (func $block_number (result i64)))

  ;; Unbounded memory: no max (needed for arbitrary-length swap paths)
  (memory (export "memory") 4)

  ;; Pool adapter interface: (pool_id, token_in_ptr, token_out_ptr, amount) -> output_amount
  (type $pool_adapter_t (func (param i32 i32 i32 i32) (result i32)))

  ;; Simple callback type: (data_ptr) -> status
  (type $callback_t (func (param i32) (result i32)))

  ;; Pool adapter vtable (each adapter handles a different pool type)
  (table 8 funcref)
  (elem (i32.const 0)
    $adapter_uniswap_v2   ;; 0: constant-product AMM
    $adapter_uniswap_v3   ;; 1: concentrated liquidity
    $adapter_curve         ;; 2: stableswap invariant
    $adapter_balancer      ;; 3: weighted pool
    $quote_uniswap_v2     ;; 4: quote for v2
    $quote_uniswap_v3     ;; 5: quote for v3
    $quote_curve           ;; 6: quote for curve
    $quote_balancer        ;; 7: quote for balancer
  )

  ;; Heap pointer for bump allocator
  (global $heap_ptr (mut i32) (i32.const 16384))

  ;; Bump allocator with memory.grow fallback
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

  ;; Memcpy utility (generates loop instruction)
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

  ;; Pool adapter: Uniswap V2 constant-product swap
  (func $adapter_uniswap_v2 (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    ;; Load reserves from storage
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (call $storage_load (local.get $out_ptr) (i32.const 1088))
    ;; x * y = k calculation (simplified: output = reserves_out * amount / (reserves_in + amount))
    (i32.store (i32.const 1152)
      (i32.div_u
        (i32.mul (i32.load (i32.const 1088)) (local.get $amount))
        (i32.add (i32.load (i32.const 1024)) (local.get $amount))
      )
    )
    ;; Update reserves
    (call $storage_store (local.get $in_ptr) (i32.const 1024))
    (call $storage_store (local.get $out_ptr) (i32.const 1088))
    (i32.load (i32.const 1152))
  )

  ;; Pool adapter: Uniswap V3 concentrated liquidity (simplified)
  (func $adapter_uniswap_v3 (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    ;; Tick-based calculation (simplified)
    (i32.store (i32.const 1152)
      (i32.div_u (i32.mul (i32.load (i32.const 1024)) (local.get $amount)) (i32.const 1000))
    )
    (call $storage_store (local.get $in_ptr) (i32.const 1024))
    (i32.load (i32.const 1152))
  )

  ;; Pool adapter: Curve stableswap
  (func $adapter_curve (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    ;; Stableswap: near 1:1 with fee
    (i32.store (i32.const 1152)
      (i32.sub (local.get $amount) (i32.div_u (local.get $amount) (i32.const 2500)))
    )
    (call $storage_store (local.get $in_ptr) (i32.const 1024))
    (i32.load (i32.const 1152))
  )

  ;; Pool adapter: Balancer weighted pool
  (func $adapter_balancer (param $pool_id i32) (param $in_ptr i32) (param $out_ptr i32) (param $amount i32) (result i32)
    (call $storage_load (local.get $in_ptr) (i32.const 1024))
    (call $storage_load (local.get $out_ptr) (i32.const 1088))
    ;; Weighted invariant (simplified)
    (i32.store (i32.const 1152)
      (i32.div_u
        (i32.mul (i32.load (i32.const 1088)) (local.get $amount))
        (i32.add (i32.load (i32.const 1024)) (local.get $amount))
      )
    )
    (call $storage_store (local.get $in_ptr) (i32.const 1024))
    (i32.load (i32.const 1152))
  )

  ;; Quote functions (read-only versions of adapters)
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

  ;; Multi-hop swap: execute a swap path of N hops through different pools
  ;; Path format: [pool_type(1B), pool_id(4B), token_in(32B), token_out(32B)] per hop
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

        ;; Compute offset for this hop: path_ptr + i * 69 (1 + 4 + 32 + 32)
        (local.set $offset
          (i32.add (local.get $path_ptr) (i32.mul (local.get $i) (i32.const 69)))
        )
        (local.set $pool_type (i32.load8_u (local.get $offset)))
        (local.set $pool_id (i32.load (i32.add (local.get $offset) (i32.const 1))))
        (local.set $token_in (i32.add (local.get $offset) (i32.const 5)))
        (local.set $token_out (i32.add (local.get $offset) (i32.const 37)))

        ;; Dynamic dispatch to appropriate pool adapter
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

  ;; Quote multi-hop: read-only price simulation
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

        ;; Dispatch to quote adapter (pool_type + 4 = quote table offset)
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

  ;; Entrypoint: route swap request
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (local $hop_count i32)
    (local $amount i32)
    (local $path_buf i32)
    (local $path_len i32)
    (local $result i32)

    (call $read_args (i32.const 0))
    (local.set $selector (i32.load (i32.const 0)))

    ;; Selector 0 = execute swap, 1 = quote
    (if (i32.le_u (local.get $selector) (i32.const 1))
      (then
        ;; Decode: hop_count(4B), amount(4B), path(N * 69B)
        (local.set $hop_count (i32.load (i32.const 4)))
        (local.set $amount (i32.load (i32.const 8)))
        (local.set $path_len (i32.mul (local.get $hop_count) (i32.const 69)))

        ;; Allocate buffer for path data
        (local.set $path_buf (call $alloc (local.get $path_len)))
        (call $memcpy (local.get $path_buf) (i32.const 12) (local.get $path_len))

        (if (i32.eqz (local.get $selector))
          (then
            ;; Execute swap
            (local.set $result
              (call $execute_swap_path
                (local.get $path_buf)
                (local.get $hop_count)
                (local.get $amount)
              )
            )
            ;; Emit swap event
            (i32.store (i32.const 2048) (local.get $result))
            (call $emit_log (i32.const 2048) (i32.const 4) (i32.const 2))
          )
          (else
            ;; Quote only
            (local.set $result
              (call $quote_swap_path
                (local.get $path_buf)
                (local.get $hop_count)
                (local.get $amount)
              )
            )
          )
        )

        ;; Return result amount
        (i32.store (i32.const 2048) (local.get $result))
        (call $write_result (i32.const 2048) (i32.const 4))
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))
)
