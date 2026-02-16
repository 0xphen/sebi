;; C++-compiled Stylus contract: simple key-value store.
;; Models typical clang/LLVM output for a basic storage contract.
;; C++ contracts use the "env" namespace for host imports.
;; Has bounded memory (max pages) and a memcpy loop but no dynamic dispatch or memory.grow.
;; Triggers: R-LOOP-01 (loop from memcpy).
;; Expected classification: RISK (exit code 1)
(module
  ;; C++ Stylus SDK host imports (clang uses "env" module)
  (import "env" "read_args" (func $read_args (param i32)))
  (import "env" "write_result" (func $write_result (param i32 i32)))
  (import "env" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "env" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  ;; Bounded memory with explicit max (typical of optimized C++ builds)
  (memory (export "memory") 2 8)

  ;; memcpy: byte-by-byte copy (clang codegen for small sizes)
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

  ;; Store a 32-byte key-value pair
  (func $kv_put (param $key_offset i32) (param $val_offset i32)
    (call $memcpy (i32.const 256) (local.get $key_offset) (i32.const 32))
    (call $memcpy (i32.const 320) (local.get $val_offset) (i32.const 32))
    (call $storage_store (i32.const 256) (i32.const 320))
  )

  ;; Load value for a 32-byte key
  (func $kv_get (param $key_offset i32)
    (call $memcpy (i32.const 256) (local.get $key_offset) (i32.const 32))
    (call $storage_load (i32.const 256) (i32.const 320))
    (call $write_result (i32.const 320) (i32.const 32))
  )

  ;; Delete: store zeroed value
  (func $kv_del (param $key_offset i32)
    (call $memcpy (i32.const 256) (local.get $key_offset) (i32.const 32))
    ;; Write 32 zero bytes
    (i64.store (i32.const 320) (i64.const 0))
    (i64.store (i32.const 328) (i64.const 0))
    (i64.store (i32.const 336) (i64.const 0))
    (i64.store (i32.const 344) (i64.const 0))
    (call $storage_store (i32.const 256) (i32.const 320))
  )

  ;; Entrypoint: selector-based dispatch
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $op i32)
    (call $read_args (i32.const 0))
    (local.set $op (i32.load8_u (i32.const 0)))

    ;; 0 = put (key at offset 1, value at offset 33)
    ;; 1 = get (key at offset 1)
    ;; 2 = delete (key at offset 1)
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
