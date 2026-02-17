(module
  (import "env" "read_args" (func $read_args (param i32)))
  (import "env" "write_result" (func $write_result (param i32 i32)))
  (import "env" "storage_load_bytes32" (func $storage_load (param i32 i32)))
  (import "env" "storage_store_bytes32" (func $storage_store (param i32 i32)))
  (import "env" "msg_sender" (func $msg_sender (param i32)))

  (memory (export "memory") 4)

  (type $vmethod (func (param i32 i32) (result i32)))

  (table 8 funcref)
  (elem (i32.const 0)
    $ERC20_name 
    $ERC20_symbol 
    $ERC20_decimals  
    $ERC20_totalSupply 
    $ERC20_balanceOf 
    $ERC20_transfer 
    $ERC20_approve 
    $ERC20_transferFrom 
  )

  (func $ERC20_name (param $this i32) (param $args i32) (result i32)
    (call $write_result (i32.const 512) (i32.const 9))
    (i32.const 0)
  )

  (func $ERC20_symbol (param $this i32) (param $args i32) (result i32)
    (call $write_result (i32.const 528) (i32.const 3))
    (i32.const 0)
  )

  (func $ERC20_decimals (param $this i32) (param $args i32) (result i32)
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
    (call $storage_load (local.get $args) (i32.const 256))
    (call $write_result (i32.const 256) (i32.const 32))
    (i32.const 0)
  )

  (func $ERC20_transfer (param $this i32) (param $args i32) (result i32)
    (call $msg_sender (i32.const 256))
    (call $storage_load (i32.const 256) (i32.const 320))
    (call $storage_store (i32.const 256) (i32.const 320))
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
    (call $storage_load (local.get $args) (i32.const 256))
    (call $storage_store (local.get $args) (i32.const 256))
    (i32.const 0)
  )

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

  (func $user_entrypoint (export "user_entrypoint") (param $args_len i32) (result i32)
    (local $selector i32)
    (local $buf i32)

    (call $read_args (i32.const 0))

    (local.set $buf (call $malloc (i32.const 1024)))

    (local.set $selector (i32.load (i32.const 0)))

    (call $decode_calldata (i32.const 4) (local.get $buf) (i32.sub (local.get $args_len) (i32.const 4)))

    (if (i32.lt_u (local.get $selector) (i32.const 8))
      (then
        (drop (call_indirect (type $vmethod)
          (i32.const 0)   
          (local.get $buf) 
          (local.get $selector)
        ))
      )
    )

    (i32.const 0)
  )

  (func (export "mark_used"))

  (data (i32.const 512) "TestToken")
  (data (i32.const 528) "TST")
)
