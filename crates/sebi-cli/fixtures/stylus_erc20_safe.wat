;; Stylus-compiled contract: minimal ERC-20 token.
;; Models a well-optimized Stylus SDK build with bounded memory, no dynamic
;; dispatch, no memory.grow, and no loops. Represents the "golden path" for
;; Stylus contracts that pass all static analysis checks.
;; Triggers: (none)
;; Expected classification: SAFE (exit code 0)
(module
  ;; Stylus VM host imports (standard Stylus ABI)
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "vm_hooks" "msg_sender" (func $msg_sender (param i32)))
  (import "vm_hooks" "emit_log" (func $emit_log (param i32 i32 i32)))

  ;; Bounded memory: 2 pages min, 16 pages max (1 MiB ceiling)
  (memory (export "memory") 2 16)

  ;; total_supply: load from storage slot 0
  (func $total_supply
    (call $storage_load (i32.const 0) (i32.const 256))
    (call $write_result (i32.const 256) (i32.const 32))
  )

  ;; balance_of: load balance for address at args offset 4
  (func $balance_of (param $addr_offset i32)
    (call $storage_load (local.get $addr_offset) (i32.const 256))
    (call $write_result (i32.const 256) (i32.const 32))
  )

  ;; transfer: move tokens from sender to recipient
  (func $transfer (param $to_offset i32) (param $amount_offset i32) (result i32)
    ;; Load sender address
    (call $msg_sender (i32.const 512))
    ;; Load sender balance
    (call $storage_load (i32.const 512) (i32.const 576))
    ;; Load recipient balance
    (call $storage_load (local.get $to_offset) (i32.const 640))

    ;; Check sender has enough: sender_bal >= amount
    (if (i32.lt_u
          (i32.load (i32.const 576))
          (i32.load (local.get $amount_offset)))
      (then (return (i32.const 1))) ;; error: insufficient balance
    )

    ;; Debit sender
    (i32.store (i32.const 576)
      (i32.sub (i32.load (i32.const 576)) (i32.load (local.get $amount_offset)))
    )
    (call $storage_store (i32.const 512) (i32.const 576))

    ;; Credit recipient
    (i32.store (i32.const 640)
      (i32.add (i32.load (i32.const 640)) (i32.load (local.get $amount_offset)))
    )
    (call $storage_store (local.get $to_offset) (i32.const 640))

    ;; Emit transfer event
    (call $emit_log (i32.const 512) (i32.const 32) (i32.const 1))

    (i32.const 0)
  )

  ;; approve: set allowance for spender
  (func $approve (param $spender_offset i32) (param $amount_offset i32)
    (call $msg_sender (i32.const 512))
    ;; Store allowance: hash(owner, spender) -> amount
    (call $storage_store (local.get $spender_offset) (local.get $amount_offset))
  )

  ;; Stylus entrypoint: 4-byte selector dispatch
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (call $read_args (i32.const 0))
    (local.set $selector (i32.load (i32.const 0)))

    ;; Dispatch based on function selector
    ;; 0 = totalSupply, 1 = balanceOf, 2 = transfer, 3 = approve
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

  ;; Static data: token metadata
  (data (i32.const 768) "StylusToken")
  (data (i32.const 800) "STK")
)
