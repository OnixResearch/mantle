# Build musl 1.2.5 using GCC 10.
#
# This is the final musl libc for the seed toolchain, built from source
# using the modern GCC compiler.
#
# Adapted from live-bootstrap steps/musl-1.2.5.

let crunch = import "lib.ncl" in

let gcc10 = import "gcc-10.ncl" in
let binutils = import "binutils-tcc.ncl" in
let make = import "make-tcc.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let stage0 = import "stage0-posix.ncl" in
let musl_old = import "musl-1.1.24-tcc-musl.ncl" in

let musl_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/musl-1.2.5/sources
  # first-consumer: bootstrap/musl-full.ncl
  url = "https://musl.libc.org/releases/musl-1.2.5.tar.gz",
  hash = "sha256-WLIl4Mb82rsOQm/nQ9TmBD067Nxqm6yNwGGDZ1jS+Y8=",
  name = "musl-1.2.5-src",
} in

{
  name = "musl-1.2.5-full",
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

      GCC=$(find_input gcc-10.5.0)
      BINUTILS=$(find_input binutils-2.30-tcc)
      MAKE=$(find_input make-3.82-tcc)
      BASH=$(find_input bash-2.05b-tcc)
      CU=$(find_input coreutils-6.10-musl)
      SED=$(find_input sed-4.0.9-musl)
      GREP=$(find_input grep-2.4-musl)
      MUSL_OLD=$(find_input musl-1.1.24-tcc-musl)
      SRC=$(find_input musl-1.2.5-src)

      export PATH="$GCC/bin:$BINUTILS/bin:$MAKE/bin:$BASH/bin:$CU/bin:$SED/bin:$GREP/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/musl-full-build
      $BB mkdir -p "$WORK"
      cd "$WORK"
      $BB cp -r "$SRC" musl-1.2.5
      $BB chmod -R u+w musl-1.2.5
      cd musl-1.2.5

      "$BASH/bin/bash" ./configure \
        --prefix="$out" \
        --syslibdir="$out/lib" \
        --host=x86_64-unknown-linux-musl \
        CC="$GCC/bin/gcc" \
        CFLAGS="-I$MUSL_OLD/include -O2" \
        LDFLAGS="-L$MUSL_OLD/lib" \
        2>&1

      make -j1 2>&1
      make -j1 install 2>&1

      test -s "$out/lib/libc.a" || { echo "ERROR: libc.a not built" >&2; exit 1; }
      test -f "$out/include/stdio.h" || { echo "ERROR: stdio.h not installed" >&2; exit 1; }
      test -f "$out/include/unistd.h" || { echo "ERROR: unistd.h not installed" >&2; exit 1; }
      test -f "$out/lib/crt1.o" || { echo "ERROR: crt1.o not installed" >&2; exit 1; }
      test -f "$out/lib/crti.o" || { echo "ERROR: crti.o not installed" >&2; exit 1; }
      test -f "$out/lib/crtn.o" || { echo "ERROR: crtn.o not installed" >&2; exit 1; }

      # Create dynamic linker symlink when libc.so is produced, and fail closed
      # if the symlink cannot be materialized.
      if [ -f "$out/lib/libc.so" ]; then
        $BB ln -sf libc.so "$out/lib/ld-musl-x86_64.so.1"
        test -e "$out/lib/ld-musl-x86_64.so.1" || { echo "ERROR: dynamic linker symlink missing" >&2; exit 1; }
      fi

      echo 'int main(void){ return 0; }' > /tmp/musl-smoke.c
      "$GCC/bin/gcc" -static -nostdinc -isystem "$out/include" /tmp/musl-smoke.c \
        "$out/lib/crt1.o" "$out/lib/crti.o" "$out/lib/libc.a" "$out/lib/crtn.o" \
        -o /tmp/musl-smoke
      /tmp/musl-smoke

      echo "musl-1.2.5 build complete"
    "%,
  ],
  inputs = [
    gcc10, binutils, make, bash, coreutils, sed, grep,
    stage0, musl_old, musl_src,
  ],
} | crunch.Derivation
