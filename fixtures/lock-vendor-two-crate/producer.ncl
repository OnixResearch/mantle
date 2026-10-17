let mantle = import "../../lib/lib.ncl" in
fun input => {
  name = "lock-vendor-producer",
  builder = "%{input.python}/bin/python3",
  system = 'x86_64-linux,
  args = [
    "%{input.bundle}/vendor-producer-build.py",
    input.bundle,
    "%{input.rust}/bin/rustc",
    "%{input.linker}/bin/cc",
  ],
  outputs = ["out"],
  inputs = [input.bundle, input.python, input.rust, input.linker] @ input.runtime,
  addressing_mode = 'input-addressed,
  sandbox = 'native,
} | mantle.Derivation
