# Build this projection first; pass its verified content-addressed out path as
# selected_table to mantle.ncl. The full mutable table is not an input of that plan.
let mantle = import "../../lib/lib.ncl" in
fun input => {
  name = "lock-vendor-selected-shared-hashes",
  builder = "%{input.producer}/bin/lock-vendor-producer",
  system = 'x86_64-linux,
  args = ["--selected-table", "%{input.bundle}/workspace/Cargo.lock", input.full_table],
  outputs = ["out"],
  inputs = [input.producer, input.bundle, input.full_table],
  addressing_mode = 'content-addressed,
  sandbox = 'native,
} | mantle.Derivation
