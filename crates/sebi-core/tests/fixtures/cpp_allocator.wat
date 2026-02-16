;; C++ Stylus contract: dynamic string handling with allocator.
;; Models a contract that manages variable-length data using new/delete.
;; Triggers: R-MEM-02 (memory.grow from allocator), R-LOOP-01 (memcpy loops).
;; Memory IS bounded, so R-MEM-01 does NOT fire.
;; Expected classification: HIGH_RISK (exit code 2, due to R-MEM-02 being HIGH)
(module
  ;; Stylus host imports
  (import "env" "read_args" (func $read_args (param i32)))
  (import "env" "write_result" (func $write_result (param i32 i32)))
  (import "env" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "env" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  ;; Bounded memory with explicit max
  (memory (export "memory") 4 64)

  ;; C++ operator new (simplified) -- calls memory.grow when heap is exhausted
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

  ;; memcpy: byte-by-byte copy (typical clang codegen for small copies)
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

  ;; memset: zero-fill buffer
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

  ;; Store a variable-length string: allocate, copy, then persist
  (func $store_string (param $data_ptr i32) (param $data_len i32) (param $slot i32)
    (local $buf i32)
    ;; Allocate buffer for the string
    (local.set $buf (call $operator_new (local.get $data_len)))
    ;; Copy string data to new buffer
    (call $memcpy (local.get $buf) (local.get $data_ptr) (local.get $data_len))
    ;; Store the first 32 bytes as a hash key
    (call $storage_store (local.get $buf) (local.get $slot))
  )

  ;; Entrypoint
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (call $read_args (i32.const 0))
    ;; Store incoming data as a string
    (call $store_string (i32.const 4) (i32.sub (local.get $args_len) (i32.const 4)) (i32.const 0))
    (i32.const 0)
  )

  (func (export "mark_used"))
)
