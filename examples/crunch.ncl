# crunch building itself.
#
# This is the self-hosting target: crunch compiled by crunch.
# Requires rustc, cargo, and build tools in the seed.
#
# Usage:
#   1. Generate extended seed:
#      crunch bootstrap -o examples/seed-rust.ncl \
#        bash coreutils gcc gnumake binutils rustc cargo
#   2. Build:
#      crunch build examples/crunch.ncl -I examples
#
# Note: This is aspirational for v0. The bwrap sandbox may need
# additional inputs (openssl-dev, protobuf, pkg-config, clang, mold)
# that aren't trivially available as single store paths. A real
# self-hosting build would use a more sophisticated build script
# or a Nickel builder package.

let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in

{
  name = "crunch",
  builder = "%{seed.bash}/bin/bash",
  args = [
    "-c",
    m%"
      set -e
      export PATH="%{seed.coreutils}/bin:%{seed.gcc}/bin:%{seed.gnumake}/bin"

      # Copy source tree into the build dir
      # (In a real build, this would be a fixed-output fetch or a source input)
      echo "crunch self-build is not yet fully functional"
      echo "This derivation demonstrates the structure."
      echo "A working self-build requires additional seed packages:"
      echo "  - rustc, cargo (Rust toolchain)"
      echo "  - protobuf (for prost-build)"
      echo "  - openssl-dev (for reqwest TLS)"
      echo "  - clang, mold (linker)"
      echo "  - pkg-config"

      mkdir -p $out/bin
      echo "#!/bin/sh" > $out/bin/crunch
      echo "echo 'placeholder: crunch self-build not yet functional'" >> $out/bin/crunch
      chmod +x $out/bin/crunch
    "%,
  ],
  inputs = [
    seed.bash,
    seed.coreutils,
    seed.gcc,
    seed.gnumake,
  ],
} | crunch.Derivation
