# mantle building itself.
#
# This is an example of the self-hosting derivation structure.
# The actual self-build uses `mantle self-build`, which generates
# a complete Nickel derivation with the full bootstrap chain
# (seed -> make -> dash -> binutils -> musl -> gcc -> busybox
# -> bwrap -> rust -> mantle). See src/self_build.rs.
#
# To prove self-hosting works:
#   cargo test -p mantle --test self_hosting -- --ignored --nocapture
#
# This example shows the structure using a Nix-based seed.
# The real self-build bypasses the seed entirely.

let mantle = import "lib.ncl" in
let seed = import "seed.ncl" in

{
  name = "mantle",
  builder = "%{seed.bash}/bin/bash",
  args = [
    "-c",
    m%"
      set -e
      export PATH="%{seed.coreutils}/bin:%{seed.gcc}/bin:%{seed.gnumake}/bin"

      # Copy source tree into the build dir
      # (In a real build, this would be a fixed-output fetch or a source input)
      echo "This example derivation demonstrates the structure."
      echo "The real self-build uses: mantle self-build"
      echo "Proof: cargo test -p mantle --test self_hosting -- --ignored"

      mkdir -p $out/bin
      echo "#!/bin/sh" > $out/bin/mantle
      echo "echo 'placeholder: mantle self-build not yet functional'" >> $out/bin/mantle
      chmod +x $out/bin/mantle
    "%,
  ],
  inputs = [
    seed.bash,
    seed.coreutils,
    seed.gcc,
    seed.gnumake,
  ],
} | mantle.Derivation
