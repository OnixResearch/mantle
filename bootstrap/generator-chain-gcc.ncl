# Shared conventional generator chain for downstream source regeneration.
#
# A normalized direct-input base avoids recursive `.inputs` flattening and the
# resulting exponential Nickel values while preserving the same derivational
# compiler/runtime/tool closure.

let base_derivation = import "gcc-generator-base.ncl" in
let grep_derivation = import "grep-2.4-gcc.ncl" in
let m4_derivation = import "m4-1.4.7-gcc.ncl" in
let gawk_derivation = import "gawk-3.0.4-gcc.ncl" in
let bison_derivation = import "bison-2.3-gcc.ncl" in

{
  base = base_derivation,
  grep = grep_derivation,
  m4 = m4_derivation,
  gawk = gawk_derivation,
  bison = bison_derivation,
}
