# Diagnostic closed-cycle GCC 4.0.4 rebuild against pass5 musl and clean binutils.
#
# GCC pass5, musl pass5, and clean binutils 2.30 form the predecessor closure.
# This rebuild removes pass4 musl from the resulting compiler and libgcc
# boundary. It remains isolated from bootstrap/seed.ncl pending repaired source
# regeneration and complete provider normalization.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let seed_gcc = crunch.derivationFile "gcc-4.0-musl-pass5.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl" in
let binutils = crunch.derivationFile "binutils-2.30-gcc.ncl" in
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
  name = "gcc-4.0.4-musl-pass6-v1",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=4.0.4
      JOB_COUNT=1
      GCC_BUILD_FLAGS="-O2 -fno-common"

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then echo "$candidate"; return; fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      SEED_GCC=$(find_input gcc-4.0.4-musl-pass5-v2)
      MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v17)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input gcc-4.0.4-native-src)
      AUTOMAKE115_SRC=$(find_input automake-1.15.1-config-sub-src)

      test -x "$SEED_GCC/bin/gcc"
      test -x "$SEED_GCC/bin/as"
      test -x "$SEED_GCC/bin/ld"
      test -x "$SEED_GCC/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -s "$SEED_GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
      test -s "$MUSL/lib/libc.a"
      test -s "$MUSL/lib/crt1.o"
      for required_tool in as ld ar ranlib readelf; do
        test -x "$BINUTILS/bin/$required_tool"
      done

      WORK=/tmp/gcc-4.0.4-musl-pass6
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
      unset MAKEFLAGS MFLAGS MAKELEVEL

      SEED_CC="$TOOLBIN/seed-gcc"
      $BB cat > "$SEED_CC" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
MANTLE_GCC_LINK_CRT="$MUSL/lib" \
exec "$SEED_GCC/bin/gcc" \
  -B"$SEED_GCC/libexec/gcc/$TARGET/$GCC_VERSION/" \
  $GCC_BUILD_FLAGS "\$@"
EOF
      $BB chmod 755 "$SEED_CC"
      export CC="$SEED_CC"
      export CXX="$SEED_CC"
      export CPP="$SEED_CC -E -I$MUSL/include"
      export CPPFLAGS="-I$MUSL/include"
      export CFLAGS="$GCC_BUILD_FLAGS -I$MUSL/include"
      export LDFLAGS="-static -L$MUSL/lib"
      export AR="$BINUTILS/bin/ar"
      export RANLIB="$BINUTILS/bin/ranlib"

      $BB mkdir -p "$out/bin"
      $BB cat > "$out/bin/as" <<EOF
#!/bin/sh
assembler="\${MANTLE_GCC_ASSEMBLER:-$BINUTILS/bin/as}"
exec "\$assembler" "\$@"
EOF
      $BB chmod 755 "$out/bin/as"
      $BB cat > "$out/bin/ld" <<EOF
#!/bin/sh
linker="\${MANTLE_GCC_LINKER:-$BINUTILS/bin/ld}"
link_arguments=
for link_argument in "\$@"; do
  case "\$link_argument" in
    -lgcc|-lgcc_eh) link_arguments="\$link_arguments $SEED_GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" ;;
    -lgcc_s) ;;
    crt1.o|crti.o|crtn.o) link_arguments="\$link_arguments $MUSL/lib/\$link_argument" ;;
    -lc) link_arguments="\$link_arguments $MUSL/lib/libc.a" ;;
    -lm) link_arguments="\$link_arguments $MUSL/lib/libm.a" ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
exec "\$linker" \$link_arguments
EOF
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
      RANLIB="$BINUTILS/bin/ranlib" \
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
        --with-as="$out/bin/as" \
        --with-ld="$out/bin/ld"

      unset MAKEFLAGS MFLAGS MAKELEVEL
      make -j"$JOB_COUNT" configure-gcc \
        CC="$SEED_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ranlib" \
        CFLAGS="$GCC_BUILD_FLAGS -I$MUSL/include" \
        MAKEINFO=true

      cd "$WORK/build/gcc"
      $BB printf '%s\n' '/*WARNING: This file is automatically generated!*/' > c-parse.y
      $BB sed -e '/^@@ifobjc.*/,/^@@end_ifobjc.*/d' \
        -e '/^@@ifc.*/d' -e '/^@@end_ifc.*/d' \
        "$WORK/src/gcc/c-parse.in" >> c-parse.y
      $BB cp "$WORK/src/gcc/c-parse.c" c-parse.c
      $BB touch c-parse.y c-parse.c
      cd "$WORK/build"

      make -j"$JOB_COUNT" all-gcc \
        CC="$SEED_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ranlib" \
        CFLAGS="$GCC_BUILD_FLAGS -I$MUSL/include" \
        BOOT_CFLAGS="$GCC_BUILD_FLAGS -I$MUSL/include" \
        MAKEINFO=true || {
          build_status=$?
          if [ -f gcc/config.log ]; then $BB cat gcc/config.log >&2; fi
          exit "$build_status"
        }
      make -j"$JOB_COUNT" install-gcc \
        CC="$SEED_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ranlib" \
        MAKEINFO=true

      echo 'gcc40-pass6-check: installed-binaries' >&2
      test -x "$out/bin/gcc"
      test -x "$out/bin/as"
      test -x "$out/bin/ld"
      test -x "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -s "$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
      "$BINUTILS/bin/ar" t "$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" > "$WORK/libgcc-members.txt"
      test -s "$WORK/libgcc-members.txt"

      $BB cat > "$out/bin/ld" <<EOF
#!/bin/sh
libc_archive="\${MANTLE_GCC_LINK_LIBC:-$MUSL/lib/libc.a}"
libm_archive="\${MANTLE_GCC_LINK_LIBM:-$MUSL/lib/libm.a}"
crt_dir="\${MANTLE_GCC_LINK_CRT:-$MUSL/lib}"
linker="\${MANTLE_GCC_LINKER:-$BINUTILS/bin/ld}"
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
echo "gcc40-pass6-final-ld: \$linker \$link_arguments" >&2
exec "\$linker" \$link_arguments
EOF
      $BB chmod 755 "$out/bin/ld"

      echo 'gcc40-pass6-check: driver-probes' >&2
      "$out/bin/gcc" -v > "$WORK/version.out" 2> "$WORK/version.err"
      "$out/bin/gcc" --help > "$WORK/help.txt"
      "$out/bin/gcc" -dumpspecs > "$WORK/specs.txt"
      test -s "$WORK/help.txt"
      test -s "$WORK/specs.txt"

      $BB cat > "$WORK/runtime-smoke.c" <<'EOF'
#define MAX_UNSIGNED_WORD 18446744073709551615ULL
#define ROUNDED_MAX_FLOAT 18446744073709551616.0f
int main(void)
{
  volatile unsigned long long value = MAX_UNSIGNED_WORD;
  float converted = (float)value;
  return converted == ROUNDED_MAX_FLOAT ? 0 : 1;
}
EOF
      echo 'gcc40-pass6-check: static-link' >&2
      "$out/bin/gcc" -I"$MUSL/include" -static "$WORK/runtime-smoke.c" -o "$WORK/runtime-smoke"
      test -s "$WORK/runtime-smoke"
      $BB chmod 755 "$WORK/runtime-smoke"
      "$WORK/runtime-smoke"
      "$BINUTILS/bin/readelf" -h "$WORK/runtime-smoke" > "$WORK/runtime-header.txt"
      $BB grep -q 'ELF' "$WORK/runtime-header.txt"

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected.o"
      if "$out/bin/gcc" -I"$MUSL/include" -c "$WORK/rejected.c" -o "$WORK/rejected.o"; then
        echo "ERROR: pass6 GCC accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB mkdir -p "$out/share/gcc-musl-pass6-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: closed GCC/musl/binutils cycle still depends on shipped generated sources and incomplete provider normalization' \
        > "$out/share/gcc-musl-pass6-frontier/NON_ADMISSION"
      echo "GCC 4.0.4 musl pass6 closed-cycle runtime checks passed; not provider admission"
    "%
  ],
  inputs = [stage0, make, seed_gcc, musl, binutils, gcc_src, automake115_src],
} | crunch.Derivation
