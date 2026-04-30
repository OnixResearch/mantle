# Rebuild musl 1.1.24 with tcc-musl (second pass).
#
# The first musl was built with tcc linked against mes libc. Now that
# tcc links against musl, rebuild musl for a self-consistent libc.
#
# Adapted from live-bootstrap steps/musl-1.1.24 (pass2).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc_musl = import "tcc-musl.ncl" in
let musl_prev = import "musl-1.1.24-tcc.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in

let musl_src = crunch.fetchTarball {
  # first-consumer: bootstrap/musl-1.1.24-tcc-musl.ncl
  # provenance: fosslinux/live-bootstrap steps/musl-1.1.24/pass2.sh source input
  url = "https://musl.libc.org/releases/musl-1.1.24.tar.gz",
  hash = "sha256-mSl3003jmxfRju1tcJlRGT75yWw13QJ89diR6M6prkE=",
  name = "musl-1.1.24-src",
} in

{
  name = "musl-1.1.24-tcc-musl",
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

      TCC=$(find_input tcc-0.9.27-musl)
      MUSL_PREV=$(find_input musl-1.1.24-tcc)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-tcc)
      MUSL_SRC=$(find_input musl-1.1.24-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/musl-rebuild
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$MUSL_SRC" musl-1.1.24
      $BB chmod -R u+w musl-1.1.24
      cd musl-1.1.24

      ./configure \
        --host=x86_64-linux-musl \
        --disable-shared \
        --prefix="$out" \
        CC=tcc \
        CFLAGS="-static -I$MUSL_PREV/include" \
        AR="tcc -ar"

      make -j1 AR="tcc -ar" RANLIB=true
      make install

      # Verify the rebuilt-musl output contract.  The later tcc-musl-v2 and
      # post-musl toolchain stages need a complete static libc install with
      # headers and startup objects, not just libc.a.
      test -f "$out/lib/libc.a"
      test -f "$out/include/stdio.h"
      if [ ! -f "$out/lib/crt1.o" ] && [ ! -f "$out/lib/Scrt1.o" ]; then
        echo "ERROR: rebuilt musl install did not produce crt1.o or Scrt1.o" >&2
        exit 1
      fi
    "%,
  ],
  inputs = [stage0, tcc_musl, musl_prev, make, sed, musl_src],
} | crunch.Derivation
