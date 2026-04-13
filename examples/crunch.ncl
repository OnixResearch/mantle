# crunch building itself.
#
# This is an example of the self-hosting derivation structure.
# The actual self-build uses `crunch self-build`, which generates
# a complete Nickel derivation with the full bootstrap chain
# (seed -> make -> dash -> binutils -> musl -> gcc -> busybox
# -> bwrap -> rust -> crunch). See src/self_build.rs.
#
# To prove self-hosting works:
#   cargo test -p crunch --test self_hosting -- --ignored --nocapture
#
# This example shows the structure using a Nix-based seed.
# The real self-build bypasses the seed entirely.

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
      echo "This example derivation demonstrates the structure."
      echo "The real self-build uses: crunch self-build"
      echo "Proof: cargo test -p crunch --test self_hosting -- --ignored"

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
