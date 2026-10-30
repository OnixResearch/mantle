(module
  (import "molten" "observe" (func $observe (param i32)))
  (import "molten" "pause" (func $pause))
  (global $total (export "total") (mut i32) (i32.const 0))
  (func (export "run")
    i32.const 7
    call $observe
    i32.const 7
    global.set $total
    call $pause
    i32.const 11
    call $observe
    global.get $total
    i32.const 11
    i32.add
    global.set $total))
