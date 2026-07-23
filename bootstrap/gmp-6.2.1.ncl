# Diagnostic GMP 6.2.1 build with the regenerated GCC 4.0 C/C++ frontier.
#
# GMP precedes GCC 4.7: the upstream GCC 4.7 compile recipe links
# -lmpc -lmpfr -lgmp. Keep this component isolated from provider selection;
# closure and normalized-provider admission remain separate evidence.

let crunch = import "lib.ncl" in

let gmp_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/gmp-6.2.1/sources
  # first-consumer: bootstrap/gmp-6.2.1.ncl
  url = "https://ftpmirror.gnu.org/gmp/gmp-6.2.1.tar.xz",
  hash = "sha256-69A2yy9AY8/OFO5FfNmsvd01vTknitC/wrQSjaZq86A=",
  name = "gmp-6.2.1-src",
} in

{
  name = "gmp-6.2.1-gcc40-v1",
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

      GCC=$(find_input gcc-4.0.4-musl-cxx-v7)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v16)
      MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      GREP=$(find_input grep-2.4-gcc-v10)
      M4TOOL=$(find_input m4-1.4.7-gcc-v5)
      SRC=$(find_input gmp-6.2.1-src)

      WORK=/tmp/gmp-6.2.1-gcc40
      TOOLBIN="$WORK/tools"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in awk basename cat chmod cmp cp cut diff dirname echo expr find head ln ls mkdir mv od printf pwd rm rmdir sed sh sleep sort tail test touch tr true uname uniq wc; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$GREP/bin:$M4TOOL/bin:$TOOLBIN:$MAKE/bin:$BINUTILS/bin:$STAGE0/bin:$PATH"
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
      CFLAGS="-O2 -fno-common -I$MUSL/include" \
      LDFLAGS="-static -L$MUSL/lib" \
        ./configure \
          --prefix="$out" \
          --build="$TARGET" \
          --host="$TARGET" \
          --disable-shared \
          --enable-static \
          --disable-assembly
      make -j1
      make -j1 install

      test -s "$out/lib/libgmp.a"
      test -s "$out/include/gmp.h"
      "$BINUTILS/bin/ar" t "$out/lib/libgmp.a" > "$WORK/archive-members.txt"
      test -s "$WORK/archive-members.txt"

      $BB cat > "$WORK/positive.c" <<EOF
#include <gmp.h>
int main(void) {
  mpz_t value;
  mpz_init_set_ui(value, $EXPECTED_VALUE);
  int matches = mpz_cmp_ui(value, $EXPECTED_VALUE) == 0;
  mpz_clear(value);
  return matches ? 0 : 1;
}
EOF
      "$CC_WRAPPER" -static -I"$out/include" -I"$MUSL/include" \
        "$WORK/positive.c" "$out/lib/libgmp.a" -o "$WORK/positive"
      test -s "$WORK/positive"
      $BB chmod 755 "$WORK/positive"
      "$WORK/positive"

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected.o"
      if "$CC_WRAPPER" -I"$out/include" -I"$MUSL/include" \
        -c "$WORK/rejected.c" -o "$WORK/rejected.o"; then
        echo "ERROR: GCC/GMP frontier accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB mkdir -p "$out/share/gmp-gcc40-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: GCC 4.0 math prerequisite has component-level runtime evidence; normalized-provider admission remains separate' \
        > "$out/share/gmp-gcc40-frontier/NON_ADMISSION"
      echo "GMP 6.2.1 GCC 4.0 runtime and rejection checks passed; not provider admission"
    "%
  ],
  inputs = [
    crunch.derivationFile "gcc-4.0-musl-cxx.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    gmp_src,
  ],
} | crunch.Derivation
