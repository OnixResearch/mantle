# Placeholder: Build minimal binutils (as, ld, ar) using tinycc.
#
# The full binutils-2.30 build requires bison, flex, m4, and many other
# tools that don't exist yet in the chain. This placeholder documents
# the dependency and will be replaced with a real build once the
# intermediate tool chain is implemented.
#
# TODO: implement the full intermediate tool chain from tcc to binutils:
#   tcc → sed → patch → gawk → tar → gzip → m4 → flex → bison → binutils-2.30
# Each step is a separate derivation adapted from live-bootstrap.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in

# TODO: add intermediate tool derivations as inputs

# Placeholder: the binutils-2.30 source tarball.
# The actual build requires many patches and intermediate tools.
let binutils_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz",
  hash = "sha256-bkZGd3uFmJA3GCGJmDQHcXgPS/oGqNMFeoSgCzFxJIQ=",
  name = "binutils-2.30-src",
} in

{
  name = "binutils-2.30-tcc",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      echo "ERROR: binutils-tcc.ncl is a placeholder." >&2
      echo "The intermediate tool chain (sed, patch, gawk, m4, flex, bison)" >&2
      echo "must be built first. See the live-bootstrap-seed-chain openspec" >&2
      echo "change for the implementation plan." >&2
      exit 1
    "%,
  ],
  inputs = [stage0, mes, tcc, binutils_src],
} | crunch.Derivation
