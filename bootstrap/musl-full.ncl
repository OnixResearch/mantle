# Placeholder: Build musl-1.2.x using GCC 10.
#
# musl libc is the final C library for the seed toolchain.
# Built from source using the from-source GCC 10.
#
# Adapted from live-bootstrap musl steps.

let crunch = import "lib.ncl" in

let gcc10 = import "gcc-10.ncl" in

let musl_src = crunch.fetchTarball {
  url = "https://musl.libc.org/releases/musl-1.2.5.tar.gz",
  hash = "sha256-m1rVnLRMPTWyaHJx/SUe2AfyikDHqmz1YPKRG32cFcE=",
  name = "musl-1.2.5-src",
} in

{
  name = "musl-1.2.5-full",
  builder = "/bin/sh",
  args = ["-c", m%"
    echo "ERROR: musl-full.ncl is a placeholder." >&2
    echo "Requires gcc-10. See live-bootstrap-seed-chain openspec." >&2
    exit 1
  "%],
  inputs = [gcc10, musl_src],
} | crunch.Derivation
