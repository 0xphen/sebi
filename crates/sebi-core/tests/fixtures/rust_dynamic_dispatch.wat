;; Rust Stylus contract: trait-object dispatch with memory growth.
;; High-risk profile: memory.grow (R-MEM-02), call_indirect (R-CALL-01).
;; Expected classification: HIGH_RISK (exit code 2)
(module
  ;; Stylus VM host imports
  (import "vm_hooks" "read_args" (func $read_args (param i32)))
  (import "vm_hooks" "write_result" (func $write_result (param i32 i32)))
  (import "vm_hooks" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "vm_hooks" "storage_store_bytes32" (func $storage_store (param i32 i32)))

  ;; Bounded memory (has max), but uses memory.grow at runtime
  (memory (export "memory") 2 256)

  ;; Function signature for trait method dispatch
  ;; Represents: fn handler(selector: i32) -> i32
  (type $handler_t (func (param i32) (result i32)))

  ;; Function table for Rust dyn Trait dispatch
  (table 4 funcref)
  (elem (i32.const 0) $handle_transfer $handle_approve $handle_balance $handle_metadata)

  ;; Trait method implementations
  (func $handle_transfer (param $arg i32) (result i32)
    (call $storage_load (i32.const 64) (i32.const 128))
    (call $storage_store (i32.const 64) (i32.const 128))
    (i32.const 0)
  )

  (func $handle_approve (param $arg i32) (result i32)
    (call $storage_store (i32.const 64) (i32.const 128))
    (i32.const 0)
  )

  (func $handle_balance (param $arg i32) (result i32)
    (call $storage_load (i32.const 64) (i32.const 128))
    (call $write_result (i32.const 128) (i32.const 32))
    (i32.const 0)
  )

  (func $handle_metadata (param $arg i32) (result i32)
    (call $write_result (i32.const 256) (i32.const 64))
    (i32.const 0)
  )

  ;; Allocator: grows memory when buffer space is needed.
  ;; This models Rust's GlobalAlloc calling memory.grow.
  (func $alloc (param $pages i32) (result i32)
    (memory.grow (local.get $pages))
  )

  ;; Stylus entrypoint with dynamic dispatch
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (call $read_args (i32.const 0))

    ;; Allocate extra page for large payloads
    (drop (call $alloc (i32.const 1)))

    ;; Read 4-byte selector from args
    (local.set $selector (i32.load (i32.const 0)))

    ;; Bounds check then dispatch via function table
    (if (i32.lt_u (local.get $selector) (i32.const 4))
      (then
        (drop (call_indirect (type $handler_t) (local.get $selector) (local.get $selector)))
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))
)
