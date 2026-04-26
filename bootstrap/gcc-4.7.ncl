# Placeholder: Build GCC 4.7.4 using GCC 4.0.4 + musl + updated tools.
#
# gcc-4.7.4 adds C++11 support needed by modern GCC.
# Between gcc-4.0.4 and gcc-4.7.4, live-bootstrap builds:
#   musl-1.1.24, updated binutils, make-4.2.1, bash, many autotools.
#
# Adapted from live-bootstrap steps/gcc-4.7.4/pass1.sh.

let crunch = import "lib.ncl" in

let gcc4 = import "gcc-4.0.ncl" in

let gcc47_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2",
  hash = "sha256-q3Oq1G/5gXGQwBYYzIhBNxDpxIqfzN5T+TlbyvOK1II=",
  name = "gcc-4.7.4-src",
} in

{
  name = "gcc-4.7.4",
  builder = "/bin/sh",
  args = ["-c", m%"
    echo "ERROR: gcc-4.7.ncl is a placeholder." >&2
    echo "Requires gcc-4.0.4 and intermediate tools. See live-bootstrap-seed-chain openspec." >&2
    exit 1
  "%],
  inputs = [gcc4, gcc47_src],
} | crunch.Derivation
