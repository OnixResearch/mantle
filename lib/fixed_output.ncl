# crunch stdlib: FixedOutput contract.
#
# Nickel record contracts may carry at most one field without a definition,
# so the shape is expressed as an open defaults contract that fills the
# optional mode, followed by the shape check for the required hash and
# algorithm.

let { HashAlgo, HashMode, Shape, .. } = import "contracts.ncl" in

let FixedOutputDefaults = {
  mode | HashMode
    | doc "Hashing mode: flat (single file) or recursive (NAR)."
    | default = 'flat,
  ..
} in
let FixedOutputRequired = {
  hash : String,
  algo : HashAlgo,
} in

{
  FixedOutput = std.contract.Sequence [
    FixedOutputDefaults,
    Shape [ "mode" ] FixedOutputRequired,
  ],
}
