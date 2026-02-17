(module
  (import "env" "read_args" (func $read_args (param i32)))
  (import "env" "write_result" (func $write_result (param i32 i32)))
  (import "env" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "env" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "env" "msg_sender" (func $msg_sender (param i32)))
  (import "env" "msg_value" (func $msg_value (param i32)))
  (import "env" "block_number" (func $block_number (result i64)))

  (memory (export "memory") 8)

  (type $bridge_op (func (param i32 i32 i32) (result i32)))

  (table 6 funcref)
  (elem (i32.const 0)
    $bridge_deposit  
    $bridge_withdraw
    $bridge_relay 
    $bridge_query_status
    $bridge_set_guardian 
    $bridge_pause
  )

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

  (func $bridge_deposit (param $this i32) (param $args i32) (param $len i32) (result i32)
    (local $amount_slot i32)
    (call $msg_sender (i32.const 512))
    (call $msg_value (i32.const 544))
    (call $storage_store (i32.const 512) (i32.const 544))
    (i64.store (i32.const 576) (call $block_number))
    (call $storage_store (i32.const 512) (i32.const 576))
    (i32.const 0)
  )

  (func $bridge_withdraw (param $this i32) (param $args i32) (param $len i32) (result i32)
    (local $buf i32)
    (local.set $buf (call $malloc (i32.const 256)))
    (call $abi_decode (local.get $args) (local.get $buf) (local.get $len))
    (call $storage_load (local.get $buf) (i32.const 640))
    (i64.store (i32.const 704) (i64.const 0))
    (i64.store (i32.const 712) (i64.const 0))
    (i64.store (i32.const 720) (i64.const 0))
    (i64.store (i32.const 728) (i64.const 0))
    (call $storage_store (local.get $buf) (i32.const 704))
    (call $write_result (i32.const 640) (i32.const 32))
    (i32.const 0)
  )

  (func $bridge_relay (param $this i32) (param $args i32) (param $len i32) (result i32)
    (local $count i32)
    (local $i i32)
    (local $proof_buf i32)

    (local.set $count (i32.load (local.get $args)))
    (local.set $proof_buf (call $malloc (i32.mul (local.get $count) (i32.const 64))))

    (local.set $i (i32.const 0))
    (block $end
      (loop $each_proof
        (br_if $end (i32.ge_u (local.get $i) (local.get $count)))
        (call $abi_decode
          (i32.add (local.get $args) (i32.add (i32.const 4) (i32.mul (local.get $i) (i32.const 64))))
          (i32.add (local.get $proof_buf) (i32.mul (local.get $i) (i32.const 64)))
          (i32.const 64)
        )
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

  (func $bridge_query_status (param $this i32) (param $args i32) (param $len i32) (result i32)
    (call $storage_load (local.get $args) (i32.const 768))
    (call $write_result (i32.const 768) (i32.const 32))
    (i32.const 0)
  )

  (func $bridge_set_guardian (param $this i32) (param $args i32) (param $len i32) (result i32)
    (call $msg_sender (i32.const 512))
    (call $storage_store (i32.const 512) (local.get $args))
    (i32.const 0)
  )

  (func $bridge_pause (param $this i32) (param $args i32) (param $len i32) (result i32)
    (call $msg_sender (i32.const 512))
    (i32.store (i32.const 832) (i32.const 1))
    (call $storage_store (i32.const 512) (i32.const 832))
    (i32.const 0)
  )

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (local $payload i32)
    (local $payload_len i32)

    (call $read_args (i32.const 0))
    (local.set $selector (i32.load (i32.const 0)))
    (local.set $payload_len (i32.sub (local.get $args_len) (i32.const 4)))

    (local.set $payload (call $malloc (local.get $payload_len)))
    (call $abi_decode (i32.const 4) (local.get $payload) (local.get $payload_len))

    (if (i32.lt_u (local.get $selector) (i32.const 6))
      (then
        (drop (call_indirect (type $bridge_op)
          (i32.const 0) 
          (local.get $payload)
          (local.get $payload_len)
          (local.get $selector)
        ))
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))

  (data (i32.const 896) "BRIDGE-V1")
)
