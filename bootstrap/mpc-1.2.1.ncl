# Build MPC 1.2.1 using GCC 4.7.4 + GMP + MPFR.
#
# MPC is the last math library prerequisite for GCC 10.
#
# Adapted from live-bootstrap steps/mpc-1.2.1.

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
let mpfr = import "mpfr-4.1.0.ncl" in
let stage0 = import "stage0-posix.ncl" in

let mpc_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/mpc/mpc-1.2.1.tar.gz",
  hash = "sha256-nyt5XHt58wWilbLRlIDuECmD3qBqmKTx4jiGL/7n5Ws=",
  name = "mpc-1.2.1-src",
} in

{
  name = "mpc-1.2.1",
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
      MPFR=$(find_input mpfr-4.1.0)
      SRC=$(find_input mpc-1.2.1-src)

      export PATH="$GCC/bin:$BINUTILS/bin:$MAKE/bin:$BASH/bin:$CU/bin:$SED/bin:$GREP/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/mpc-build
      $BB mkdir -p "$WORK"
      cd "$WORK"
      $BB cp -r "$SRC" mpc-1.2.1
      $BB chmod -R u+w mpc-1.2.1
      cd mpc-1.2.1

      "$BASH/bin/bash" ./configure \
        --prefix="$out" \
        --build=x86_64-unknown-linux-musl \
        --host=x86_64-unknown-linux-musl \
        --disable-shared \
        --enable-static \
        --with-gmp="$GMP" \
        --with-mpfr="$MPFR" \
        CC="$GCC/bin/gcc" \
        CFLAGS="-I$MUSL/include -I$GMP/include -I$MPFR/include -D_GNU_SOURCE -O2" \
        LDFLAGS="-static -L$MUSL/lib -L$GMP/lib -L$MPFR/lib" \
        2>&1

      make -j1 2>&1
      make -j1 install 2>&1

      test -f "$out/lib/libmpc.a" || { echo "ERROR: libmpc.a not built" >&2; exit 1; }
      echo "mpc-1.2.1 build complete"
    "%,
  ],
  inputs = [
    gcc47, binutils, musl, make, bash, coreutils, sed, grep,
    gmp, mpfr, stage0, mpc_src,
  ],
} | crunch.Derivation
