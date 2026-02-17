(module
  (import "env" "memory" (memory 2))

  (func $echo (export "echo") (param $offset i32) (param $len i32) (result i32)
    (local.get $offset)
  )

  (func (export "mark_used"))
)
