# Placeholder: Build binutils-2.41 using GCC 10.
#
# Full binutils for the seed toolchain: as, ld, ar, nm, objcopy, etc.
#
# Adapted from live-bootstrap steps/binutils-2.41.

let crunch = import "lib.ncl" in

let gcc10 = import "gcc-10.ncl" in

let binutils_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.41.tar.xz",
  hash = "sha256-rppXieI0WeWWBuZalksFOLB+MmN1HNb1y+n0HGNX2MA=",
  name = "binutils-2.41-src",
} in

{
  name = "binutils-2.41-full",
  builder = "/bin/sh",
  args = ["-c", m%"
    echo "ERROR: binutils-full.ncl is a placeholder." >&2
    echo "Requires gcc-10. See live-bootstrap-seed-chain openspec." >&2
    exit 1
  "%],
  inputs = [gcc10, binutils_src],
} | crunch.Derivation
