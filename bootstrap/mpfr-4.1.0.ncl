# Diagnostic MPFR 4.1.0 build with regenerated GCC 4.0 and source-built GMP.
# This math component precedes GCC 4.7 and remains isolated from provider
# selection; closure and normalized-provider admission remain separate evidence.

let crunch = import "lib.ncl" in

let mpfr_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/mpfr-4.1.0/sources
  # first-consumer: bootstrap/mpfr-4.1.0.ncl
  url = "https://ftpmirror.gnu.org/mpfr/mpfr-4.1.0.tar.xz",
  hash = "sha256-Q8xRKVAvE1c0uPSWy/Kav0Hfw0UxlnWzePXE13u6sDw=",
  name = "mpfr-4.1.0-src",
} in

{
  name = "mpfr-4.1.0-gcc40-v1",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=4.0.4
      EXPECTED_VALUE=42

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then echo "$candidate"; return; fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      GCC=$(find_input gcc-4.0.4-musl-cxx-v8)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v17)
      MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      GREP=$(find_input grep-2.4-gcc-v10)
      GMP=$(find_input gmp-6.2.1-gcc40-v1)
      SRC=$(find_input mpfr-4.1.0-src)

      WORK=/tmp/mpfr-4.1.0-gcc40
      TOOLBIN="$WORK/tools"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in awk basename cat chmod cmp cp cut diff dirname echo expr find head ln ls mkdir mv od printf pwd rm rmdir sed sh sleep sort tail test touch tr true uname uniq wc; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$GREP/bin:$TOOLBIN:$MAKE/bin:$BINUTILS/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      unset MAKEFLAGS MFLAGS MAKELEVEL

      CC_WRAPPER="$TOOLBIN/gcc40"
      $BB cat > "$CC_WRAPPER" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
MANTLE_GCC_LINK_LIBGCC="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
MANTLE_GCC_LINK_CRT="$MUSL/lib" \
exec "$GCC/bin/gcc" \
  -B"$GCC/bin/" \
  -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
  "\$@"
EOF
      $BB chmod 755 "$CC_WRAPPER"

      $BB cp -R "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"
      CC="$CC_WRAPPER" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ranlib" \
      CFLAGS="-O2 -fno-common -I$MUSL/include -I$GMP/include" \
      LDFLAGS="-static -L$MUSL/lib -L$GMP/lib" \
        ./configure \
          --prefix="$out" \
          --build="$TARGET" \
          --host="$TARGET" \
          --disable-shared \
          --enable-static \
          --with-gmp="$GMP"
      make -j1
      make -j1 install

      test -s "$out/lib/libmpfr.a"
      test -s "$out/include/mpfr.h"
      test -s "$out/include/mpf2mpfr.h"
      "$BINUTILS/bin/ar" t "$out/lib/libmpfr.a" > "$WORK/archive-members.txt"
      test -s "$WORK/archive-members.txt"

      $BB cat > "$WORK/positive.c" <<EOF
#include <mpfr.h>
int main(void) {
  mpfr_t value;
  mpfr_init2(value, 128);
  mpfr_set_ui(value, $EXPECTED_VALUE, MPFR_RNDN);
  int matches = mpfr_cmp_ui(value, $EXPECTED_VALUE) == 0;
  mpfr_clear(value);
  return matches ? 0 : 1;
}
EOF
      "$CC_WRAPPER" -static -I"$out/include" -I"$GMP/include" -I"$MUSL/include" \
        "$WORK/positive.c" "$out/lib/libmpfr.a" "$GMP/lib/libgmp.a" \
        -o "$WORK/positive"
      test -s "$WORK/positive"
      $BB chmod 755 "$WORK/positive"
      "$WORK/positive"

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected.o"
      if "$CC_WRAPPER" -I"$out/include" -I"$GMP/include" -I"$MUSL/include" \
        -c "$WORK/rejected.c" -o "$WORK/rejected.o"; then
        echo "ERROR: GCC/MPFR frontier accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB mkdir -p "$out/share/mpfr-gcc40-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: GCC 4.0 math prerequisite has component-level runtime evidence; normalized-provider admission remains separate' \
        > "$out/share/mpfr-gcc40-frontier/NON_ADMISSION"
      echo "MPFR 4.1.0 GCC 4.0 runtime and rejection checks passed; not provider admission"
    "%
  ],
  inputs = [
    crunch.derivationFile "gcc-4.0-musl-cxx.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "gmp-6.2.1.ncl",
    mpfr_src,
  ],
} | crunch.Derivation
