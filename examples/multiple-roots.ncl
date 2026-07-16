# Build two independent top-level roots from one Nickel file.
#
# Usage:
#   mantle build examples/multiple-roots.ncl --no-substitute

let mantle = import "lib.ncl" in
let message = fun derivation_name text => {
  name = derivation_name,
  builder = "/bin/sh",
  args = ["-c", "printf '%s\\n' '%{text}' > \"$out\""],
} | mantle.Derivation in

{
  alpha = message "multiple-roots-alpha" "alpha root built independently",
  beta = message "multiple-roots-beta" "beta root built independently",
}
