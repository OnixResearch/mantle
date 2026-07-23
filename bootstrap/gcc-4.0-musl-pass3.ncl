# Diagnostic GCC 4.0.4 rebuild against the GCC-built musl successor.
#
# This stage tests whether the pass2 compiler can produce a conventional GCC
# compiler plus libgcc using source GAS/ar/ld and the full upstream musl build.
# It remains isolated from bootstrap/seed.ncl pending complete provider checks.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let seed_gcc = crunch.derivationFile "gcc-4.0-native.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-gcc.ncl" in
let binutils = crunch.derivationFile "binutils-2.30-gas-source.ncl" in
let gcc_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2",
  hash = "sha256-QNHCUqWDcOtqVLokzAd3MfsWWR/jN3HdAld/Pm+GSM0=",
  name = "gcc-4.0.4-native-src",
} in
let automake115_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/automake/automake-1.15.1.tar.xz",
  hash = "sha256-/KHwBUWoEgUxjLuLFOpqM4c0UXUDWiyiBV4HBTwOcZU=",
  name = "automake-1.15.1-config-sub-src",
} in

{
  name = "gcc-4.0.4-musl-pass3-v22",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=4.0.4
      JOB_COUNT=1
      GCC_COMPAT_FLAGS="-fno-asynchronous-unwind-tables -fno-unwind-tables -fno-common"

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then echo "$candidate"; return; fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      SEED_GCC=$(find_input gcc-4.0.4-native-gas-v45)
      MUSL=$(find_input musl-1.1.24-gcc-pass3-v5)
      BINUTILS=$(find_input binutils-2.30-gas-source-v41)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input gcc-4.0.4-native-src)
      AUTOMAKE115_SRC=$(find_input automake-1.15.1-config-sub-src)

      WORK=/tmp/gcc-4.0.4-musl-pass3
      TOOLBIN="$WORK/tools"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in awk basename cat chmod cmp cp cut dirname echo egrep expr find grep head ln ls mkdir mv od printf pwd rm rmdir sed sh sort tail tar test touch tr true uname uniq wc; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$TOOLBIN:$BINUTILS/bin:$MAKE/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export MAKEINFO=true

      $BB cat > "$TOOLBIN/as" <<EOF
#!/bin/sh
assembler_arguments=
for assembler_argument in "\$@"; do
  case "\$assembler_argument" in
    -Qy) ;;
    *) assembler_arguments="\$assembler_arguments \$assembler_argument" ;;
  esac
done
exec "$BINUTILS/bin/as" \$assembler_arguments
EOF
      $BB chmod 755 "$TOOLBIN/as"

      $BB cat > "$TOOLBIN/ld" <<EOF
#!/bin/sh
link_arguments=
for link_argument in "\$@"; do
  case "\$link_argument" in
    -lgcc|-lgcc_eh|-lgcc_s|*/libgcc.a|*/libtcc1.a|*/crtbegin*.o|*/crtend*.o|crtbegin*.o|crtend*.o) ;;
    crt1.o|crti.o|crtn.o) link_arguments="\$link_arguments $MUSL/lib/\$link_argument" ;;
    -lc) link_arguments="\$link_arguments $MUSL/lib/libc.a" ;;
    -lm) link_arguments="\$link_arguments $MUSL/lib/libm.a" ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
exec "$BINUTILS/bin/ld" \$link_arguments "$WORK/seed-link-compat.o"
EOF
      $BB chmod 755 "$TOOLBIN/ld"

      SEED_CC="$TOOLBIN/seed-gcc"
      $BB cat > "$SEED_CC" <<EOF
#!/bin/sh
exec "$SEED_GCC/bin/gcc" \
  -B"$SEED_GCC/libexec/gcc/$TARGET/$GCC_VERSION/" \
  -B"$TOOLBIN/" \
  $GCC_COMPAT_FLAGS "\$@"
EOF
      $BB chmod 755 "$SEED_CC"
      export CC="$SEED_CC"
      export CXX="$SEED_CC"
      export CPP="$SEED_CC -E -I$MUSL/include"
      export CPPFLAGS="-I$MUSL/include"
      export CFLAGS="$GCC_COMPAT_FLAGS -I$MUSL/include"
      export LDFLAGS="-static -L$MUSL/lib"
      export AR="$BINUTILS/bin/ar"
      export RANLIB="$BINUTILS/bin/ar s"

      $BB cat > "$WORK/seed-link-compat.c" <<'EOF'
typedef union {
  struct { unsigned int low; int high; } words;
  long long signed_value;
  unsigned long long unsigned_value;
} mantle_double_word;

float __floatundisf(unsigned long long value)
{
  mantle_double_word bits;
  long double converted;
  bits.unsigned_value = value;
  if (bits.words.high >= 0) return (float) bits.signed_value;
  converted = (long double) bits.signed_value;
  converted += 18446744073709551616.0;
  return (float) converted;
}
EOF
      "$SEED_CC" -c "$WORK/seed-link-compat.c" -o "$WORK/seed-link-compat.o"
      test -s "$WORK/seed-link-compat.o"

      $BB mkdir -p "$out/bin"
      $BB cp "$TOOLBIN/ld" "$out/bin/ld"
      $BB chmod 755 "$out/bin/ld"

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
      test -f configure || { echo "ERROR: GCC configure missing" >&2; exit 1; }
      test -f gcc/c-parse.c || { echo "ERROR: shipped C parser missing" >&2; exit 1; }
      test -f gcc/gengtype-yacc.c || { echo "ERROR: shipped gengtype parser missing" >&2; exit 1; }
      test -f gcc/gengtype-yacc.h || { echo "ERROR: shipped gengtype parser header missing" >&2; exit 1; }
      test -f gcc/gengtype-lex.c || { echo "ERROR: shipped gengtype lexer missing" >&2; exit 1; }
      $BB touch gcc/c-parse.c gcc/gengtype-yacc.c gcc/gengtype-yacc.h gcc/gengtype-lex.c

      for config_sub in $($BB find . -name config.sub -type f); do
        $BB cp "$AUTOMAKE115_SRC/lib/config.sub" "$config_sub"
        $BB chmod 755 "$config_sub"
      done
      $BB rm -rf "$WORK/build"
      $BB mkdir -p "$WORK/build"
      cd "$WORK/build"

      CC="$SEED_CC" \
      CXX="$SEED_CC" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ar s" \
      "$WORK/src/configure" \
        --prefix="$out" \
        --libdir="$out/lib" \
        --build="$TARGET" \
        --host="$TARGET" \
        --target="$TARGET" \
        --enable-languages=c \
        --disable-bootstrap \
        --disable-multilib \
        --disable-nls \
        --disable-shared \
        --disable-libssp \
        --disable-libmudflap \
        --with-system-zlib=no \
        --with-sysroot="$MUSL" \
        --with-native-system-header-dir=/include \
        --with-as="$BINUTILS/bin/as" \
        --with-ld="$out/bin/ld"

      unset MAKEFLAGS MFLAGS MAKELEVEL
      make -j"$JOB_COUNT" configure-gcc \
        CC="$SEED_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ar s" \
        CFLAGS="$GCC_COMPAT_FLAGS -I$MUSL/include" \
        MAKEINFO=true

      cd "$WORK/build/gcc"
      $BB printf '%s\n' '/*WARNING: This file is automatically generated!*/' > c-parse.y
      $BB sed -e '/^@@ifobjc.*/,/^@@end_ifobjc.*/d' \
        -e '/^@@ifc.*/d' -e '/^@@end_ifc.*/d' \
        "$WORK/src/gcc/c-parse.in" >> c-parse.y
      $BB cp "$WORK/src/gcc/c-parse.c" c-parse.c
      $BB touch c-parse.y c-parse.c
      $BB cp "$TOOLBIN/as" as
      $BB chmod 755 as
      cd "$WORK/build"

      make -j"$JOB_COUNT" all-gcc \
        CC="$SEED_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ar s" \
        CFLAGS="$GCC_COMPAT_FLAGS -I$MUSL/include" \
        BOOT_CFLAGS="$GCC_COMPAT_FLAGS -I$MUSL/include" \
        MAKEINFO=true || {
          build_status=$?
          if [ -f gcc/config.log ]; then $BB cat gcc/config.log >&2; fi
          exit "$build_status"
        }
      make -j"$JOB_COUNT" install-gcc \
        CC="$SEED_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ar s" \
        MAKEINFO=true
      echo 'gcc40-pass3-check: installed-binaries' >&2
      test -x "$out/bin/gcc"
      test -x "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -s "$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
      echo 'gcc40-pass3-check: libgcc-archive' >&2
      "$BINUTILS/bin/ar" t "$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" > "$WORK/libgcc-members.txt"
      test -s "$WORK/libgcc-members.txt"

      echo 'gcc40-pass3-check: final-driver-setup' >&2
      FINAL_TOOLBIN="$WORK/final-tools"
      $BB mkdir -p "$FINAL_TOOLBIN"
      $BB cp "$TOOLBIN/as" "$FINAL_TOOLBIN/as"
      $BB cat > "$out/bin/ld" <<EOF
#!/bin/sh
libc_archive="\${MANTLE_GCC_LINK_LIBC:-$MUSL/lib/libc.a}"
libm_archive="\${MANTLE_GCC_LINK_LIBM:-$MUSL/lib/libm.a}"
crt_dir="\${MANTLE_GCC_LINK_CRT:-$MUSL/lib}"
link_arguments=
for link_argument in "\$@"; do
  case "\$link_argument" in
    -lgcc|-lgcc_eh) link_arguments="\$link_arguments $out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" ;;
    -lgcc_s) ;;
    crt1.o|crti.o|crtn.o) link_arguments="\$link_arguments \$crt_dir/\$link_argument" ;;
    -lc) link_arguments="\$link_arguments \$libc_archive" ;;
    -lm) link_arguments="\$link_arguments \$libm_archive" ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
echo "gcc40-pass3-final-ld: $BINUTILS/bin/ld \$link_arguments" >&2
exec "$BINUTILS/bin/ld" \$link_arguments
EOF
      $BB chmod 755 "$out/bin/ld"
      $BB cp "$out/bin/ld" "$FINAL_TOOLBIN/ld"
      $BB chmod 755 "$FINAL_TOOLBIN/as" "$FINAL_TOOLBIN/ld"

      FINAL_CC="$FINAL_TOOLBIN/gcc"
      $BB cat > "$FINAL_CC" <<EOF
#!/bin/sh
exec "$out/bin/gcc" \
  -B"$out/libexec/gcc/$TARGET/$GCC_VERSION/" \
  -B"$FINAL_TOOLBIN/" \
  $GCC_COMPAT_FLAGS "\$@"
EOF
      $BB chmod 755 "$FINAL_CC"
      echo 'gcc40-pass3-check: final-driver-probes' >&2
      "$FINAL_CC" -v > "$WORK/final-version.out" 2> "$WORK/final-version.err"
      "$FINAL_CC" --help > "$WORK/final-help.txt"
      "$FINAL_CC" -dumpspecs > "$WORK/final-specs.txt"
      test -s "$WORK/final-help.txt"
      test -s "$WORK/final-specs.txt"

      $BB cat > "$WORK/final-smoke.c" <<'EOF'
#define MAX_UNSIGNED_WORD 18446744073709551615ULL
#define ROUNDED_MAX_FLOAT 18446744073709551616.0f
int main(void)
{
  volatile unsigned long long value = MAX_UNSIGNED_WORD;
  float converted = (float)value;
  return converted == ROUNDED_MAX_FLOAT ? 0 : 1;
}
EOF
      echo 'gcc40-pass3-check: final-static-link' >&2
      "$FINAL_CC" -I"$MUSL/include" -static "$WORK/final-smoke.c" -o "$WORK/final-smoke"
      test -s "$WORK/final-smoke"
      $BB chmod 755 "$WORK/final-smoke"
      "$WORK/final-smoke"
      "$BINUTILS/bin/readelf" -h "$WORK/final-smoke" > "$WORK/final-header.txt"
      $BB grep -q 'ELF' "$WORK/final-header.txt"

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected.o"
      if "$FINAL_CC" -I"$MUSL/include" -c "$WORK/rejected.c" -o "$WORK/rejected.o"; then
        echo "ERROR: rebuilt GCC accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB mkdir -p "$out/share/gcc-musl-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: GCC/musl pass3 still depends on reduced source-binutils behavior and unrepaired regeneration tools' \
        > "$out/share/gcc-musl-frontier/NON_ADMISSION"
      echo "GCC 4.0.4 musl pass3 compiler/libgcc runtime checks passed; not provider admission"
    "%
  ],
  inputs = [stage0, make, seed_gcc, musl, binutils, gcc_src, automake115_src],
} | crunch.Derivation
