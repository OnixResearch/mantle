# Build GMP 6.2.1 using GCC 4.7.4.
#
# GMP is a prerequisite for MPFR and MPC, which GCC 10 requires.
#
# Adapted from live-bootstrap steps/gmp-6.2.1.

let crunch = import "lib.ncl" in

let gcc47 = import "gcc-4.7.ncl" in
let binutils = import "binutils-tcc.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let stage0 = import "stage0-posix.ncl" in

let gmp_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gmp/gmp-6.2.1.tar.xz",
  hash = "sha256-69A2yy9AY8/OFO5FfNmsvd01vTknitC/wrQSjaZq86A=",
  name = "gmp-6.2.1-src",
} in

{
  name = "gmp-6.2.1",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for d in $NIX_STORE/*-"$1"; do
          if [ -d "$d" ]; then echo "$d"; return; fi
        done
        echo "ERROR: input $1 not found" >&2; exit 1
      }

      GCC=$(find_input gcc-4.7.4)
      BINUTILS=$(find_input binutils-2.30-tcc)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      BASH=$(find_input bash-2.05b-tcc)
      CU=$(find_input coreutils-6.10-musl)
      SED=$(find_input sed-4.0.9-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      GAWK=$(find_input gawk-3.0.4-musl)
      M4=$(find_input m4-1.4.7-musl)
      SRC=$(find_input gmp-6.2.1-src)

      export PATH="$GCC/bin:$BINUTILS/bin:$MAKE/bin:$BASH/bin:$CU/bin:$SED/bin:$GREP/bin:$DIFF/bin:$GAWK/bin:$M4/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/gmp-build
      $BB mkdir -p "$WORK"
      cd "$WORK"
      $BB cp -r "$SRC" gmp-6.2.1
      $BB chmod -R u+w gmp-6.2.1
      cd gmp-6.2.1

      "$BASH/bin/bash" ./configure \
        --prefix="$out" \
        --build=x86_64-unknown-linux-musl \
        --host=x86_64-unknown-linux-musl \
        --disable-shared \
        --enable-static \
        CC="$GCC/bin/gcc" \
        CFLAGS="-I$MUSL/include -D_GNU_SOURCE -O2" \
        LDFLAGS="-static -L$MUSL/lib" \
        2>&1

      make -j1 2>&1
      make -j1 install 2>&1

      test -f "$out/lib/libgmp.a" || { echo "ERROR: libgmp.a not built" >&2; exit 1; }
      echo "gmp-6.2.1 build complete"
    "%,
  ],
  inputs = [
    gcc47, binutils, musl, make, bash, coreutils, sed, grep,
    diffutils, gawk, m4, stage0, gmp_src,
  ],
} | crunch.Derivation
