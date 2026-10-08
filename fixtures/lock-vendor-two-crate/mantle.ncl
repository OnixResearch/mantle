# The producer is built by producer.ncl from the declared pure core and shell source.
# Supply the resulting store output and source/toolchain store roots at invocation.
let mantle = import "../../lib/lib.ncl" in
fun input => {
  name = "lock-vendor-two-crate-plan",
  builder = "%{input.producer}/bin/lock-vendor-producer",
  system = 'x86_64-linux,
  args = [
    "%{input.bundle}/workspace/Cargo.lock",
    input.selected_table,
    "%{input.python}/bin/python3",
    input.bundle,
    input.rust,
  ] @ input.runtime,
  outputs = ["plan"],
  dynamic_plan_outputs = ["plan"],
  inputs = [input.producer, input.bundle, input.selected_table, input.python, input.rust] @ input.runtime,
  addressing_mode = 'input-addressed,
  sandbox = 'native,
} | mantle.Derivation
