# Build MPFR 4.1.0 using GCC 4.7.4 + GMP.
#
# MPFR is a prerequisite for MPC, which GCC 10 requires.
#
# Adapted from live-bootstrap steps/mpfr-4.1.0.

let crunch = import "lib.ncl" in

let gcc47 = import "gcc-4.7.ncl" in
let binutils = import "binutils-tcc.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let gmp = import "gmp-6.2.1.ncl" in
let stage0 = import "stage0-posix.ncl" in

let mpfr_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/mpfr/mpfr-4.1.0.tar.xz",
  hash = "sha256-MFoqCFj5ihkVOBn2vaMRjVENRKJW1ziAr4+Pyih6mCE=",
  name = "mpfr-4.1.0-src",
} in

{
  name = "mpfr-4.1.0",
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
      GMP=$(find_input gmp-6.2.1)
      SRC=$(find_input mpfr-4.1.0-src)

      export PATH="$GCC/bin:$BINUTILS/bin:$MAKE/bin:$BASH/bin:$CU/bin:$SED/bin:$GREP/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/mpfr-build
      $BB mkdir -p "$WORK"
      cd "$WORK"
      $BB cp -r "$SRC" mpfr-4.1.0
      $BB chmod -R u+w mpfr-4.1.0
      cd mpfr-4.1.0

      "$BASH/bin/bash" ./configure \
        --prefix="$out" \
        --build=x86_64-unknown-linux-musl \
        --host=x86_64-unknown-linux-musl \
        --disable-shared \
        --enable-static \
        --with-gmp="$GMP" \
        CC="$GCC/bin/gcc" \
        CFLAGS="-I$MUSL/include -I$GMP/include -D_GNU_SOURCE -O2" \
        LDFLAGS="-static -L$MUSL/lib -L$GMP/lib" \
        2>&1

      make -j1 2>&1
      make -j1 install 2>&1

      test -f "$out/lib/libmpfr.a" || { echo "ERROR: libmpfr.a not built" >&2; exit 1; }
      echo "mpfr-4.1.0 build complete"
    "%,
  ],
  inputs = [
    gcc47, binutils, musl, make, bash, coreutils, sed, grep,
    gmp, stage0, mpfr_src,
  ],
} | crunch.Derivation
