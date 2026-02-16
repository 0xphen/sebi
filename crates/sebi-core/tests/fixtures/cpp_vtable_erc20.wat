;; C++ Stylus contract: ERC-20 token with virtual dispatch.
;; Models typical clang/LLVM output for a class hierarchy with virtual methods.
;; Triggers: R-MEM-01 (no max), R-MEM-02 (memory.grow), R-CALL-01 (call_indirect), R-LOOP-01 (loop).
;; Expected classification: HIGH_RISK (exit code 2)
(module
  ;; C++ Stylus SDK host imports (typically under "env" module)
  (import "env" "read_args" (func $read_args (param i32)))
  (import "env" "write_result" (func $write_result (param i32 i32)))
  (import "env" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "env" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "env" "msg_sender" (func $msg_sender (param i32)))

  ;; C++ contracts typically have unbounded memory (no max pages)
  (memory (export "memory") 4)

  ;; Virtual function table type: (this_ptr, args_ptr) -> status
  ;; This is how clang compiles C++ virtual methods
  (type $vmethod (func (param i32 i32) (result i32)))

  ;; Virtual dispatch table (C++ vtable laid out in function table)
  (table 8 funcref)
  (elem (i32.const 0)
    $ERC20_name            ;; selector 0
    $ERC20_symbol          ;; selector 1
    $ERC20_decimals        ;; selector 2
    $ERC20_totalSupply     ;; selector 3
    $ERC20_balanceOf       ;; selector 4
    $ERC20_transfer        ;; selector 5
    $ERC20_approve         ;; selector 6
    $ERC20_transferFrom    ;; selector 7
  )

  ;; ERC-20 virtual method implementations
  (func $ERC20_name (param $this i32) (param $args i32) (result i32)
    ;; Return "TestToken" from static data area
    (call $write_result (i32.const 512) (i32.const 9))
    (i32.const 0)
  )

  (func $ERC20_symbol (param $this i32) (param $args i32) (result i32)
    (call $write_result (i32.const 528) (i32.const 3))
    (i32.const 0)
  )

  (func $ERC20_decimals (param $this i32) (param $args i32) (result i32)
    ;; Store 18 as a uint8 result
    (i32.store8 (i32.const 256) (i32.const 18))
    (call $write_result (i32.const 256) (i32.const 1))
    (i32.const 0)
  )

  (func $ERC20_totalSupply (param $this i32) (param $args i32) (result i32)
    (call $storage_load (i32.const 0) (i32.const 256))
    (call $write_result (i32.const 256) (i32.const 32))
    (i32.const 0)
  )

  (func $ERC20_balanceOf (param $this i32) (param $args i32) (result i32)
    ;; Load balance from storage[keccak(addr)]
    (call $storage_load (local.get $args) (i32.const 256))
    (call $write_result (i32.const 256) (i32.const 32))
    (i32.const 0)
  )

  (func $ERC20_transfer (param $this i32) (param $args i32) (result i32)
    ;; Load sender balance, verify, update, store
    (call $msg_sender (i32.const 256))
    (call $storage_load (i32.const 256) (i32.const 320))
    (call $storage_store (i32.const 256) (i32.const 320))
    ;; Load recipient balance, update, store
    (call $storage_load (local.get $args) (i32.const 384))
    (call $storage_store (local.get $args) (i32.const 384))
    (i32.const 0)
  )

  (func $ERC20_approve (param $this i32) (param $args i32) (result i32)
    (call $msg_sender (i32.const 256))
    (call $storage_store (i32.const 256) (local.get $args))
    (i32.const 0)
  )

  (func $ERC20_transferFrom (param $this i32) (param $args i32) (result i32)
    ;; Check allowance, transfer, update allowance
    (call $storage_load (local.get $args) (i32.const 256))
    (call $storage_store (local.get $args) (i32.const 256))
    (i32.const 0)
  )

  ;; C++ heap allocator (dlmalloc pattern - uses memory.grow)
  (func $malloc (param $size i32) (result i32)
    (local $pages i32)
    (local.set $pages
      (i32.div_u
        (i32.add (local.get $size) (i32.const 65535))
        (i32.const 65536)
      )
    )
    (i32.mul (memory.grow (local.get $pages)) (i32.const 65536))
  )

  ;; ABI decoder loop: copies calldata bytes into structured buffer
  (func $decode_calldata (param $src i32) (param $dst i32) (param $len i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $done
      (loop $copy
        (br_if $done (i32.ge_u (local.get $i) (local.get $len)))
        (i32.store8
          (i32.add (local.get $dst) (local.get $i))
          (i32.load8_u (i32.add (local.get $src) (local.get $i)))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $copy)
      )
    )
  )

  ;; Main entrypoint: decode selector, allocate buffer, vtable dispatch
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (local $buf i32)

    (call $read_args (i32.const 0))

    ;; Allocate a result buffer via malloc
    (local.set $buf (call $malloc (i32.const 1024)))

    ;; Read 4-byte function selector
    (local.set $selector (i32.load (i32.const 0)))

    ;; Decode remaining calldata
    (call $decode_calldata (i32.const 4) (local.get $buf) (i32.sub (local.get $args_len) (i32.const 4)))

    ;; Bounds-checked vtable dispatch (C++ virtual call)
    (if (i32.lt_u (local.get $selector) (i32.const 8))
      (then
        (drop (call_indirect (type $vmethod)
          (i32.const 0)           ;; this pointer
          (local.get $buf)        ;; decoded args
          (local.get $selector)   ;; table index
        ))
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))

  ;; Static data: token name and symbol
  (data (i32.const 512) "TestToken")
  (data (i32.const 528) "TST")
)
