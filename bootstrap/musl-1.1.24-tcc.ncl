# Build musl 1.1.24 using the tcc-musl-prep compiler.
#
# This is the first real libc in the chain, replacing the mes libc.
# After this, tools can link against musl instead of mes libc.
#
# Adapted from live-bootstrap steps/musl-1.1.24 (pass1).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc_prep = import "tcc-musl-prep.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in

let musl_src = crunch.fetchTarball {
  url = "https://musl.libc.org/releases/musl-1.1.24.tar.gz",
  hash = "sha256-E3DJqBKyzyp9koAlEMygBYzDfmanvt1wBR8KNAFQIqM=",
  name = "musl-1.1.24-src",
} in

{
  name = "musl-1.1.24-tcc",
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

      TCC_PREP=$(find_input tcc-0.9.27-musl-prep)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      MES_OUT=$(find_input mes)
      SED=$(find_input sed-4.0.9-tcc)
      MUSL_SRC=$(find_input musl-1.1.24-src)

      export PATH="$TCC_PREP/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      WORK=/tmp/musl-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$MUSL_SRC" musl-1.1.24
      $BB chmod -R u+w musl-1.1.24
      cd musl-1.1.24

      # Configure musl for static build with tcc
      ./configure \
        --host=x86_64-linux-musl \
        --disable-shared \
        --prefix="$out" \
        CC=tcc \
        CFLAGS="-static -D__DEFINED_struct_timespec" \
        AR="tcc -ar"

      # Build
      make -j1 AR="tcc -ar" RANLIB=true

      # Install headers, static lib, and crt files
      make install

      # Verify
      test -f "$out/lib/libc.a"
      test -f "$out/include/stdio.h"
      test -f "$out/lib/crt1.o" || test -f "$out/lib/Scrt1.o" || true
    "%,
  ],
  inputs = [stage0, mes, tcc_prep, make, sed, musl_src],
} | crunch.Derivation
