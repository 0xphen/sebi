;; Rust-compiled Stylus contract: simple counter with get/increment/decrement.
;; Models typical `rustc --target wasm32-unknown-unknown` output via Stylus SDK.
;; Safe profile: bounded memory (max pages), no memory.grow, no call_indirect, no loops.
;; Expected classification: SAFE (exit code 0)
(module
  ;; Stylus VM host imports (Rust SDK uses the "vm_hooks" namespace)
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  ;; Bounded memory: 1 min page, 4 max pages (256 KiB ceiling)
  (memory (export "memory") 1 4)

  ;; Counter storage slot (32 zero bytes = slot 0)
  (data (i32.const 0) "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00")

  ;; Load current counter value into memory at offset 64
  (func $get_counter
    (call $storage_load (i32.const 0) (i32.const 64))
    (call $write_result (i32.const 64) (i32.const 32))
  )

  ;; Increment: load value, add 1, store back
  (func $increment
    (call $storage_load (i32.const 0) (i32.const 64))
    (i32.store
      (i32.const 64)
      (i32.add (i32.load (i32.const 64)) (i32.const 1))
    )
    (call $storage_store (i32.const 0) (i32.const 64))
  )

  ;; Decrement: load value, subtract 1, store back
  (func $decrement
    (call $storage_load (i32.const 0) (i32.const 64))
    (i32.store
      (i32.const 64)
      (i32.sub (i32.load (i32.const 64)) (i32.const 1))
    )
    (call $storage_store (i32.const 0) (i32.const 64))
  )

  ;; Stylus user_entrypoint: selector dispatch
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (call $read_args (i32.const 128))
    (local.set $selector (i32.load8_u (i32.const 128)))

    ;; 0 = get, 1 = increment, 2 = decrement
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
