;; Rust Stylus contract: simple storage getter/setter.
;; Safe profile: bounded memory, no memory.grow, no call_indirect, no loops.
;; Expected classification: SAFE (exit code 0)
(module
  ;; Stylus VM host imports (typical for Rust SDK)
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  ;; Bounded memory with explicit maximum (2 min, 16 max pages)
  (memory (export "memory") 2 16)

  ;; Storage getter: reads a 32-byte key and returns the value
  (func $get (param $key_ptr i32)
    (call $storage_load (local.get $key_ptr) (i32.const 128))
    (call $write_result (i32.const 128) (i32.const 32))
  )

  ;; Storage setter: reads a 64-byte (key + value) payload and stores it
  (func $set (param $payload_ptr i32)
    (call $storage_store (local.get $payload_ptr) (i32.add (local.get $payload_ptr) (i32.const 32)))
  )

  ;; Stylus user entrypoint
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    ;; Read args into linear memory at offset 0
    (call $read_args (i32.const 0))

    ;; Dispatch: first byte is selector (0 = get, 1 = set)
    (if (i32.eqz (i32.load8_u (i32.const 0)))
      (then (call $get (i32.const 1)))
      (else (call $set (i32.const 1)))
    )

    ;; Return success
    (i32.const 0)
  )

  ;; Stylus activation marker
  (func (export "mark_used"))
)
