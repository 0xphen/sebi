;; Rust Stylus contract: array iterator with unbounded memory.
;; Risky profile: has loops (R-LOOP-01), no memory max (R-MEM-01).
;; Expected classification: RISK (exit code 1)
(module
  ;; Stylus VM host imports
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  ;; Unbounded memory -- no maximum declared
  (memory (export "memory") 2)

  ;; Batch storage writer: iterates over N key-value pairs and stores each one.
  ;; This is a common Rust pattern from Vec::iter().for_each() or a for loop.
  (func $batch_store (param $base i32) (param $count i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $break
      (loop $continue
        ;; Exit when all items processed
        (br_if $break (i32.ge_u (local.get $i) (local.get $count)))
        ;; Compute key offset: base + i * 64
        ;; Compute value offset: base + i * 64 + 32
        (call $storage_store
          (i32.add (local.get $base) (i32.mul (local.get $i) (i32.const 64)))
          (i32.add
            (i32.add (local.get $base) (i32.mul (local.get $i) (i32.const 64)))
            (i32.const 32)
          )
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )
  )

  ;; Batch storage reader: loads N values and copies them into a result buffer.
  (func $batch_load (param $keys_base i32) (param $out_base i32) (param $count i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $done
      (loop $again
        (br_if $done (i32.ge_u (local.get $i) (local.get $count)))
        (call $storage_load
          (i32.add (local.get $keys_base) (i32.mul (local.get $i) (i32.const 32)))
          (i32.add (local.get $out_base) (i32.mul (local.get $i) (i32.const 32)))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $again)
      )
    )
  )

  ;; Stylus entrypoint
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (call $read_args (i32.const 0))
    ;; First byte: operation (0 = batch_store, 1 = batch_load)
    ;; Second 4 bytes: count
    (if (i32.eqz (i32.load8_u (i32.const 0)))
      (then
        (call $batch_store
          (i32.const 5)
          (i32.load (i32.const 1))
        )
      )
      (else
        (call $batch_load
          (i32.const 5)
          (i32.const 4096)
          (i32.load (i32.const 1))
        )
        (call $write_result (i32.const 4096) (i32.mul (i32.load (i32.const 1)) (i32.const 32)))
      )
    )
    (i32.const 0)
  )

  (func (export "mark_used"))
)
