# Diagnostic musl rebuild using the GCC 4.0 compiler and libgcc rebuilt
# against the preceding GCC-built musl sysroot.
#
# This stage remains isolated from bootstrap/seed.ncl until the complete
# compiler/libc/binutils/provider admission surface and regeneration chain pass.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let gcc = crunch.derivationFile "gcc-4.0-musl-pass3.ncl" in
let binutils = crunch.derivationFile "binutils-2.30-gas-source.ncl" in
let musl_src = crunch.derivationFile "musl-1.1.24-src.ncl" in

{
  name = "musl-1.1.24-gcc-pass4-v2",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=4.0.4
      JOB_COUNT=1
      MUSL_CFLAGS="-O2 -fno-common"

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then
            echo "$candidate"
            return
          fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      GCC=$(find_input gcc-4.0.4-musl-pass3-v22)
      BINUTILS=$(find_input binutils-2.30-gas-source-v41)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input musl-1.1.24-src)

      test -x "$GCC/bin/gcc"
      test -x "$GCC/bin/ld"
      test -x "$GCC/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -s "$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
      test -x "$BINUTILS/bin/as"
      test -x "$BINUTILS/bin/ar"
      test -x "$BINUTILS/bin/ld"
      test -x "$BINUTILS/bin/readelf"

      WORK=/tmp/musl-gcc-pass4
      TOOLBIN="$WORK/tools"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in awk basename cat chmod cp dirname echo expr find grep head ln mkdir mv od printf pwd rm rmdir sed sh sort test touch tr true uname wc; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$TOOLBIN:$BINUTILS/bin:$MAKE/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      unset MAKEFLAGS MFLAGS MAKELEVEL

      $BB cp -r "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"
      if [ ! -f configure ]; then
        for configure_path in */configure; do
          if [ -f "$configure_path" ]; then
            cd "${configure_path%/configure}"
            break
          fi
        done
      fi
      test -f configure || { echo "ERROR: musl configure script missing" >&2; exit 1; }

      CC="$GCC/bin/gcc" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ar s" \
      ./configure \
        --host="$TARGET" \
        --disable-shared \
        --prefix="$out" \
        --libdir="$out/lib" \
        --includedir="$out/include"

      make -j"$JOB_COUNT" \
        CROSS_COMPILE= \
        CC="$GCC/bin/gcc" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ar s" \
        CFLAGS_AUTO= \
        CFLAGS="$MUSL_CFLAGS"
      make install

      $BB mkdir -p "$out/usr"
      $BB ln -s ../include "$out/usr/include"
      $BB ln -s ../lib "$out/usr/lib"

      test -s "$out/lib/libc.a"
      test -s "$out/lib/crt1.o"
      test -s "$out/lib/crti.o"
      test -s "$out/lib/crtn.o"
      test -f "$out/include/semaphore.h"
      test -f "$out/usr/include/semaphore.h"
      test -s "$out/usr/lib/libc.a"
      "$BINUTILS/bin/ar" t "$out/lib/libc.a" > "$WORK/libc-members.txt"
      test -s "$WORK/libc-members.txt"

      $BB cat > "$WORK/runtime-smoke.c" <<'EOF'
#include <semaphore.h>
#include <stdio.h>
#include <unistd.h>

#define MAX_UNSIGNED_WORD 18446744073709551615ULL
#define ROUNDED_MAX_FLOAT 18446744073709551616.0f

enum runtime_failure {
  FAILURE_PAGE_SIZE = 11,
  FAILURE_SEMAPHORE_INIT = 12,
  FAILURE_SEMAPHORE_WAIT = 13,
  FAILURE_SEMAPHORE_POST = 14,
  FAILURE_SEMAPHORE_VALUE = 15,
  FAILURE_SEMAPHORE_DESTROY = 16,
  FAILURE_STDIO = 17,
  FAILURE_UNSIGNED_CONVERSION = 18
};

int main(void)
{
  sem_t semaphore;
  int value = 0;
  volatile unsigned long long unsigned_value = MAX_UNSIGNED_WORD;
  float converted = (float)unsigned_value;
  if (getpagesize() <= 0) return FAILURE_PAGE_SIZE;
  if (sem_init(&semaphore, 0, 1) != 0) return FAILURE_SEMAPHORE_INIT;
  if (sem_wait(&semaphore) != 0) return FAILURE_SEMAPHORE_WAIT;
  if (sem_post(&semaphore) != 0) return FAILURE_SEMAPHORE_POST;
  if (sem_getvalue(&semaphore, &value) != 0 || value != 1) return FAILURE_SEMAPHORE_VALUE;
  if (sem_destroy(&semaphore) != 0) return FAILURE_SEMAPHORE_DESTROY;
  if (converted != ROUNDED_MAX_FLOAT) return FAILURE_UNSIGNED_CONVERSION;
  if (fputs("gcc-pass4-musl-smoke-ok\n", stdout) < 0) return FAILURE_STDIO;
  return 0;
}
EOF
      MANTLE_GCC_LINK_LIBC="$out/lib/libc.a" \
      MANTLE_GCC_LINK_LIBM="$out/lib/libm.a" \
      MANTLE_GCC_LINK_CRT="$out/lib" \
      "$GCC/bin/gcc" -I"$out/include" -static \
        "$WORK/runtime-smoke.c" -o "$WORK/runtime-smoke"
      test -s "$WORK/runtime-smoke"
      $BB chmod 755 "$WORK/runtime-smoke"
      smoke_output=$("$WORK/runtime-smoke")
      test "$smoke_output" = gcc-pass4-musl-smoke-ok
      "$BINUTILS/bin/readelf" -h "$WORK/runtime-smoke" > "$WORK/runtime-header.txt"
      $BB grep -q 'ELF' "$WORK/runtime-header.txt"

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected.o"
      if "$GCC/bin/gcc" -I"$out/include" -c "$WORK/rejected.c" -o "$WORK/rejected.o"; then
        echo "ERROR: pass4 GCC/musl stage accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB mkdir -p "$out/share/musl-gcc-pass4-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: pass4 musl still depends on reduced source-binutils behavior and unrepaired regeneration tools' \
        > "$out/share/musl-gcc-pass4-frontier/NON_ADMISSION"
      echo "musl-1.1.24 GCC pass4 runtime checks passed; not provider admission"
    "%
  ],
  inputs = [stage0, make, gcc, binutils, musl_src],
} | crunch.Derivation
