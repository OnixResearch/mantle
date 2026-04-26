# Placeholder: Build GCC 4.0.4 using tinycc + make + binutils.
#
# gcc-4.0.4 is the oldest GCC that tinycc can build. It is the entry
# point into the GCC version ladder.
#
# Prerequisites (not yet implemented):
#   tcc → sed → patch → gawk → tar → gzip → m4 → flex → bison
#   → binutils-2.30 → gcc-4.0.4
#
# Adapted from live-bootstrap steps/gcc-4.0.4/pass1.sh.

let crunch = import "lib.ncl" in

let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in
let binutils = import "binutils-tcc.ncl" in

let gcc_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2",
  hash = "sha256-kJLkxw84mjCJeH/VHgVVVfWq8LtTa4nRwHjy+e/hdf0=",
  name = "gcc-4.0.4-src",
} in

{
  name = "gcc-4.0.4",
  builder = "/bin/sh",
  args = ["-c", m%"
    echo "ERROR: gcc-4.0.ncl is a placeholder." >&2
    echo "Requires the intermediate tool chain. See live-bootstrap-seed-chain openspec." >&2
    exit 1
  "%],
  inputs = [tcc, make, binutils, gcc_src],
} | crunch.Derivation
