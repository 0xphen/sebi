;; C++-compiled Stylus contract: cross-chain token bridge with escrow and relay.
;; Models a complex C++ contract with class hierarchies compiled by clang/LLVM.
;; Features: vtable dispatch for bridge operations, dlmalloc allocator, ABI
;; decoding loops, and unbounded memory typical of unoptimized C++ builds.
;; Triggers: R-MEM-01 (no max), R-MEM-02 (memory.grow), R-CALL-01 (call_indirect), R-LOOP-01 (loop).
;; Expected classification: HIGH_RISK (exit code 2)
(module
  ;; C++ Stylus SDK host imports ("env" module)
  (import "env" "read_args" (func $read_args (param i32)))
  (import "env" "write_result" (func $write_result (param i32 i32)))
  (import "env" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "env" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "env" "msg_sender" (func $msg_sender (param i32)))
  (import "env" "msg_value" (func $msg_value (param i32)))
  (import "env" "block_number" (func $block_number (result i64)))

  ;; Unbounded memory (no max) -- common in C++ builds with dynamic allocation
  (memory (export "memory") 8)

  ;; Virtual method type: (this_ptr, args_ptr, args_len) -> status
  (type $bridge_op (func (param i32 i32 i32) (result i32)))

  ;; Vtable for bridge operations (C++ virtual dispatch)
  (table 6 funcref)
  (elem (i32.const 0)
    $bridge_deposit      ;; 0: lock tokens on source chain
    $bridge_withdraw     ;; 1: release tokens on dest chain
    $bridge_relay        ;; 2: relay proof from validator
    $bridge_query_status ;; 3: query bridge transfer status
    $bridge_set_guardian ;; 4: admin: set guardian address
    $bridge_pause        ;; 5: admin: pause bridge
  )

  ;; dlmalloc-style allocator (typical C++ codegen)
  (func $malloc (param $size i32) (result i32)
    (local $pages i32)
    (local $base i32)
    (local.set $pages
      (i32.div_u
        (i32.add (local.get $size) (i32.const 65535))
        (i32.const 65536)
      )
    )
    (local.set $base (memory.grow (local.get $pages)))
    (if (i32.eq (local.get $base) (i32.const -1))
      (then (unreachable))
    )
    (i32.mul (local.get $base) (i32.const 65536))
  )

  ;; ABI decoder: copies calldata into structured buffer (byte loop)
  (func $abi_decode (param $src i32) (param $dst i32) (param $len i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $done
      (loop $decode
        (br_if $done (i32.ge_u (local.get $i) (local.get $len)))
        (i32.store8
          (i32.add (local.get $dst) (local.get $i))
          (i32.load8_u (i32.add (local.get $src) (local.get $i)))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $decode)
      )
    )
  )

  ;; Deposit: lock tokens in escrow, record pending transfer
  (func $bridge_deposit (param $this i32) (param $args i32) (param $len i32) (result i32)
    (local $amount_slot i32)
    (call $msg_sender (i32.const 512))
    (call $msg_value (i32.const 544))
    ;; Store deposit record: sender -> amount
    (call $storage_store (i32.const 512) (i32.const 544))
    ;; Record block number as timestamp
    (i64.store (i32.const 576) (call $block_number))
    (call $storage_store (i32.const 512) (i32.const 576))
    (i32.const 0)
  )

  ;; Withdraw: verify proof and release tokens
  (func $bridge_withdraw (param $this i32) (param $args i32) (param $len i32) (result i32)
    (local $buf i32)
    ;; Allocate verification buffer
    (local.set $buf (call $malloc (i32.const 256)))
    ;; Decode withdrawal proof into buffer
    (call $abi_decode (local.get $args) (local.get $buf) (local.get $len))
    ;; Load escrow record
    (call $storage_load (local.get $buf) (i32.const 640))
    ;; Zero out escrow (release)
    (i64.store (i32.const 704) (i64.const 0))
    (i64.store (i32.const 712) (i64.const 0))
    (i64.store (i32.const 720) (i64.const 0))
    (i64.store (i32.const 728) (i64.const 0))
    (call $storage_store (local.get $buf) (i32.const 704))
    (call $write_result (i32.const 640) (i32.const 32))
    (i32.const 0)
  )

  ;; Relay: validator submits proof batch
  (func $bridge_relay (param $this i32) (param $args i32) (param $len i32) (result i32)
    (local $count i32)
    (local $i i32)
    (local $proof_buf i32)

    ;; First 4 bytes = number of proofs
    (local.set $count (i32.load (local.get $args)))
    (local.set $proof_buf (call $malloc (i32.mul (local.get $count) (i32.const 64))))

    ;; Decode each proof entry
    (local.set $i (i32.const 0))
    (block $end
      (loop $each_proof
        (br_if $end (i32.ge_u (local.get $i) (local.get $count)))
        (call $abi_decode
          (i32.add (local.get $args) (i32.add (i32.const 4) (i32.mul (local.get $i) (i32.const 64))))
          (i32.add (local.get $proof_buf) (i32.mul (local.get $i) (i32.const 64)))
          (i32.const 64)
        )
        ;; Store each proof in storage
        (call $storage_store
          (i32.add (local.get $proof_buf) (i32.mul (local.get $i) (i32.const 64)))
          (i32.add (local.get $proof_buf) (i32.add (i32.mul (local.get $i) (i32.const 64)) (i32.const 32)))
        )
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $each_proof)
      )
    )
    (i32.const 0)
  )

  ;; Query transfer status
  (func $bridge_query_status (param $this i32) (param $args i32) (param $len i32) (result i32)
    (call $storage_load (local.get $args) (i32.const 768))
    (call $write_result (i32.const 768) (i32.const 32))
    (i32.const 0)
  )

  ;; Admin: set guardian address
  (func $bridge_set_guardian (param $this i32) (param $args i32) (param $len i32) (result i32)
    (call $msg_sender (i32.const 512))
    ;; Store new guardian: admin_slot -> new_guardian
    (call $storage_store (i32.const 512) (local.get $args))
    (i32.const 0)
  )

  ;; Admin: pause bridge
  (func $bridge_pause (param $this i32) (param $args i32) (param $len i32) (result i32)
    (call $msg_sender (i32.const 512))
    ;; Set pause flag in storage
    (i32.store (i32.const 832) (i32.const 1))
    (call $storage_store (i32.const 512) (i32.const 832))
    (i32.const 0)
  )

  ;; Main entrypoint: decode selector, allocate, vtable dispatch
  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (local $payload i32)
    (local $payload_len i32)

    (call $read_args (i32.const 0))
    (local.set $selector (i32.load (i32.const 0)))
    (local.set $payload_len (i32.sub (local.get $args_len) (i32.const 4)))

    ;; Allocate payload buffer via malloc
    (local.set $payload (call $malloc (local.get $payload_len)))
    (call $abi_decode (i32.const 4) (local.get $payload) (local.get $payload_len))

    ;; Bounds-checked C++ virtual dispatch
    (if (i32.lt_u (local.get $selector) (i32.const 6))
      (then
        (drop (call_indirect (type $bridge_op)
          (i32.const 0)              ;; this pointer
          (local.get $payload)       ;; decoded args
          (local.get $payload_len)   ;; args length
          (local.get $selector)      ;; vtable index
        ))
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))

  ;; Static data
  (data (i32.const 896) "BRIDGE-V1")
)
