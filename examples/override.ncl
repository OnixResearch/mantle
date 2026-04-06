# Demonstrate overrideAttrs: customize a package without rewriting it.
#
# Usage:
#   1. Generate seed: crunch bootstrap -o examples/seed.ncl
#   2. Eval: crunch eval examples/override.ncl -I examples -I builders

let crunch = import "lib.ncl" in
let builders = import "builders/lib.ncl" in
let seed = import "seed.ncl" in
let stdenv = builders.mkStdenv seed in

let hello = stdenv.mkDerivation {
  pname = "hello",
  version = "1.0",
  buildPhase = m%"
    echo '#!/bin/sh' > hello
    echo 'echo "Hello"' >> hello
    chmod +x hello
  "%,
  installPhase = m%"
    mkdir -p $out/bin
    cp hello $out/bin/
  "%,
} in

# Override: change the greeting without touching the original definition.
# The function receives old attrs and returns ONLY the fields to patch.
hello.overrideAttrs (fun old => {
  version = "1.0-debug",
  buildPhase = m%"
    echo '#!/bin/sh' > hello
    echo 'echo "Hello (debug build)"' >> hello
    chmod +x hello
  "%,
})
