# Placeholder: Build GCC 10.5.0 using GCC 4.7.4.
#
# gcc-10.5.0 is the modern GCC that produces the final seed toolchain.
# Requires gmp, mpfr, mpc libraries built with gcc-4.7.4.
#
# Adapted from live-bootstrap steps/gcc-10.5.0/pass1.sh.

let crunch = import "lib.ncl" in

let gcc47 = import "gcc-4.7.ncl" in

let gcc10_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-10.5.0/gcc-10.5.0.tar.xz",
  hash = "sha256-JBIJ24Wq7Jnl8KHfRf7Q2Bv01VNbptVxOP+drvrJJjM=",
  name = "gcc-10.5.0-src",
} in

{
  name = "gcc-10.5.0",
  builder = "/bin/sh",
  args = ["-c", m%"
    echo "ERROR: gcc-10.ncl is a placeholder." >&2
    echo "Requires gcc-4.7.4 + gmp + mpfr + mpc. See live-bootstrap-seed-chain openspec." >&2
    exit 1
  "%],
  inputs = [gcc47, gcc10_src],
} | crunch.Derivation
