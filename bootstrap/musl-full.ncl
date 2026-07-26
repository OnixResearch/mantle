# Final-native musl 1.2.5 static/dynamic runtime built by receipted GCC 10.
#
# This output supplies the libc/header/CRT/interpreter portion of the bounded
# final row and remains separate from provider or seed admission.

let crunch = import "lib.ncl" in

let musl_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/musl-1.2.5/sources
  # first-consumer: bootstrap/musl-full.ncl
  url = "https://musl.libc.org/releases/musl-1.2.5.tar.gz",
  hash = "sha256-WLIl4Mb82rsOQm/nQ9TmBD067Nxqm6yNwGGDZ1jS+Y8=",
  name = "musl-1.2.5-src",
} in

{
  name = "musl-1.2.5-gcc10-v1",
  builder = "/bin/sh",
  # The dynamic interpreter path is part of produced executables.
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=10.5.0
      JOB_COUNT=1
      MUSL_CFLAGS='-O2 -fno-common'
      WORK=/tmp/musl-1.2.5-gcc10
      TOOLBIN="$WORK/tools"

      find_input() {
        input_suffix=$1
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            echo "$input_path"
            return 0
          fi
        done
        echo "ERROR: input $input_suffix not found" >&2
        exit 1
      }

      GCC=$(find_input gcc-10.5.0-musl-gcc47-v2)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v16)
      MUSL_OLD=$(find_input musl-1.1.24-gcc-pass5-v1)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      GREP=$(find_input grep-2.4-gcc-v10)
      SRC=$(find_input musl-1.2.5-src)

      test -x "$GCC/bin/gcc"
      test -x "$GCC/bin/as"
      test -x "$GCC/bin/ld"
      test -x "$GCC/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -s "$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
      for required_tool in as ld ar ranlib readelf; do
        test -x "$BINUTILS/bin/$required_tool"
      done

      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in awk basename cat chmod cmp cp cut dirname echo env expr false find grep head install ln ls mkdir mv od printf pwd readlink rm rmdir sed sh sleep sort tail test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
      $BB ln -sf "$MAKE/bin/make" "$TOOLBIN/make"

      CLEAN_CC="$TOOLBIN/gcc10"
      $BB cat > "$CLEAN_CC" <<EOF
#!/bin/sh
export MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as"
export MANTLE_GCC_LINKER="$BINUTILS/bin/ld"
export MANTLE_GCC_LINK_LIBC="$MUSL_OLD/lib/libc.a"
export MANTLE_GCC_LINK_LIBM="$MUSL_OLD/lib/libm.a"
export MANTLE_GCC_LINK_LIBGCC="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
export MANTLE_GCC_LINK_CRT="$MUSL_OLD/lib"
exec "$GCC/bin/gcc" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" "\$@"
EOF
      $BB chmod 755 "$CLEAN_CC"

      export PATH="$TOOLBIN:$BINUTILS/bin:$STAGE0/bin"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export MAKEFLAGS=

      $BB cp -R "$SRC" "$WORK/src"
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
      test -f configure

      CC="$CLEAN_CC" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ranlib" \
      ./configure \
        --target="$TARGET" \
        --prefix="$out" \
        --syslibdir="$out/lib" \
        --enable-shared \
        CFLAGS="$MUSL_CFLAGS"

      make -j"$JOB_COUNT" \
        CROSS_COMPILE= \
        CC="$CLEAN_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ranlib"
      make install \
        CROSS_COMPILE= \
        CC="$CLEAN_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ranlib"

      $BB mkdir -p "$out/usr"
      $BB ln -s ../include "$out/usr/include"
      $BB ln -s ../lib "$out/usr/lib"

      test -s "$out/lib/libc.a"
      test -s "$out/lib/libc.so"
      test -s "$out/lib/crt1.o"
      test -s "$out/lib/Scrt1.o"
      test -s "$out/lib/rcrt1.o"
      test -s "$out/lib/crti.o"
      test -s "$out/lib/crtn.o"
      test -f "$out/include/semaphore.h"
      test -f "$out/usr/include/semaphore.h"
      test -s "$out/usr/lib/libc.a"
      test -e "$out/lib/ld-musl-x86_64.so.1"
      "$BINUTILS/bin/ar" t "$out/lib/libc.a" > "$WORK/libc-members.txt"
      test -s "$WORK/libc-members.txt"
      "$BINUTILS/bin/readelf" -h "$out/lib/libc.so" | "$GREP/bin/grep" -q 'DYN'

      $BB cat > "$WORK/runtime-smoke.c" <<'EOF'
#include <semaphore.h>
#include <stdio.h>
#include <unistd.h>

enum runtime_failure {
  FAILURE_PAGE_SIZE = 11,
  FAILURE_SEMAPHORE_INIT = 12,
  FAILURE_SEMAPHORE_WAIT = 13,
  FAILURE_SEMAPHORE_POST = 14,
  FAILURE_SEMAPHORE_VALUE = 15,
  FAILURE_SEMAPHORE_DESTROY = 16,
  FAILURE_STDIO = 17
};

int main(void)
{
  sem_t semaphore;
  int value = 0;
  if (getpagesize() <= 0) return FAILURE_PAGE_SIZE;
  if (sem_init(&semaphore, 0, 1) != 0) return FAILURE_SEMAPHORE_INIT;
  if (sem_wait(&semaphore) != 0) return FAILURE_SEMAPHORE_WAIT;
  if (sem_post(&semaphore) != 0) return FAILURE_SEMAPHORE_POST;
  if (sem_getvalue(&semaphore, &value) != 0 || value != 1) return FAILURE_SEMAPHORE_VALUE;
  if (sem_destroy(&semaphore) != 0) return FAILURE_SEMAPHORE_DESTROY;
  if (fputs("gcc10-musl-1.2.5-smoke-ok\n", stdout) < 0) return FAILURE_STDIO;
  return 0;
}
EOF

      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
        "$GCC/bin/gcc" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
          -I"$out/include" -c "$WORK/runtime-smoke.c" \
          -o "$WORK/runtime-smoke.o"
      test -s "$WORK/runtime-smoke.o"

      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
      MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
      MANTLE_GCC_LINK_LIBC="$out/lib/libc.a" \
      MANTLE_GCC_LINK_LIBM="$out/lib/libm.a" \
      MANTLE_GCC_LINK_LIBGCC="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
      MANTLE_GCC_LINK_CRT="$out/lib" \
        "$GCC/bin/gcc" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
          -nostartfiles -static \
          "$out/lib/crt1.o" "$out/lib/crti.o" \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION/crtbeginT.o" \
          "$WORK/runtime-smoke.o" \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION/crtend.o" "$out/lib/crtn.o" \
          -o "$WORK/runtime-static"
      test -s "$WORK/runtime-static"
      $BB chmod 755 "$WORK/runtime-static"
      static_output=$("$WORK/runtime-static")
      test "$static_output" = gcc10-musl-1.2.5-smoke-ok
      "$BINUTILS/bin/readelf" -h "$WORK/runtime-static" | "$GREP/bin/grep" -q 'ELF64'

      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
      MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
      MANTLE_GCC_LINK_LIBC="$out/lib/libc.so" \
      MANTLE_GCC_LINK_LIBM="$out/lib/libc.so" \
      MANTLE_GCC_LINK_LIBGCC="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
      MANTLE_GCC_LINK_CRT="$out/lib" \
        "$GCC/bin/gcc" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
          -nostartfiles \
          -Wl,-dynamic-linker,"$out/lib/ld-musl-x86_64.so.1" \
          "$out/lib/crt1.o" "$out/lib/crti.o" \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION/crtbegin.o" \
          "$WORK/runtime-smoke.o" \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION/crtend.o" "$out/lib/crtn.o" \
          -o "$WORK/runtime-dynamic"
      test -s "$WORK/runtime-dynamic"
      $BB chmod 755 "$WORK/runtime-dynamic"
      dynamic_output=$("$WORK/runtime-dynamic")
      test "$dynamic_output" = gcc10-musl-1.2.5-smoke-ok
      "$BINUTILS/bin/readelf" -l "$WORK/runtime-dynamic" | "$GREP/bin/grep" -q "$out/lib/ld-musl-x86_64.so.1"

      RELOCATED_MUSL="$WORK/relocated-musl"
      $BB cp -R "$out" "$RELOCATED_MUSL"
      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
        "$GCC/bin/gcc" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
          -I"$RELOCATED_MUSL/include" -c "$WORK/runtime-smoke.c" \
          -o "$WORK/runtime-relocated.o"
      test -s "$WORK/runtime-relocated.o"
      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
      MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
      MANTLE_GCC_LINK_LIBC="$RELOCATED_MUSL/lib/libc.a" \
      MANTLE_GCC_LINK_LIBM="$RELOCATED_MUSL/lib/libm.a" \
      MANTLE_GCC_LINK_LIBGCC="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
      MANTLE_GCC_LINK_CRT="$RELOCATED_MUSL/lib" \
        "$GCC/bin/gcc" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
          -nostartfiles -static \
          "$RELOCATED_MUSL/lib/crt1.o" "$RELOCATED_MUSL/lib/crti.o" \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION/crtbeginT.o" \
          "$WORK/runtime-relocated.o" \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION/crtend.o" "$RELOCATED_MUSL/lib/crtn.o" \
          -o "$WORK/runtime-relocated-static"
      test -s "$WORK/runtime-relocated-static"
      $BB chmod 755 "$WORK/runtime-relocated-static"
      relocated_static_output=$("$WORK/runtime-relocated-static")
      test "$relocated_static_output" = gcc10-musl-1.2.5-smoke-ok

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected.o"
      if MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
        "$GCC/bin/gcc" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
          -I"$out/include" -c "$WORK/rejected.c" -o "$WORK/rejected.o"; then
        echo "ERROR: GCC 10/musl 1.2.5 accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/final-native-musl-row.txt" <<'EOF'
claim: bounded final-native musl 1.2.5 matrix passed
positive: headers,CRT,static libc,shared libc,dynamic interpreter; static,dynamic,and copied-tree runtime
negative: malformed C rejected without output
non_claims: general libc correctness; provider admission; seed admission; whole-bootstrap correctness
EOF
      echo "musl 1.2.5 final-native static/dynamic runtime and rejection matrix passed"
    "%
  ],
  inputs = [
    crunch.derivationFile "gcc-10.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    musl_src,
  ],
} | crunch.Derivation
