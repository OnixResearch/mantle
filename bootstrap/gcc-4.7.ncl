# Diagnostic GCC 4.7.4 C/C++ build over regenerated GCC 4.0 plus the
# source-built GMP/MPFR/MPC prerequisite chain.
#
# This replaces the previous dependency cycle that attempted GCC 4.7 before
# the math libraries its upstream compile recipe links. The build regenerates
# its bounded Autoconf, header, Bison, and Flex inputs before compilation.

let crunch = import "lib.ncl" in

let gcc47_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/gcc-4.7.4/sources
  # first-consumer: bootstrap/gcc-4.7.ncl
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2",
  hash = "sha256-7ffdeULrk9WbESRDdRXHT1DoOvrThEBIbozmder5xTc=",
  name = "gcc-4.7.4-src",
} in
let automake115_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/automake/automake-1.15.1.tar.xz",
  hash = "sha256-/KHwBUWoEgUxjLuLFOpqM4c0UXUDWiyiBV4HBTwOcZU=",
  name = "automake-1.15.1-config-sub-src",
} in

{
  name = "gcc-4.7.4-musl-gcc40-v1",
  builder = "/bin/sh",
  # GCC embeds its configured prefix. Input addressing keeps that prefix equal
  # to the published logical output path instead of a CA provisional.
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      SEED_GCC_VERSION=4.0.4
      OUTPUT_GCC_VERSION=4.7.4
      JOB_COUNT=1
      BUILD_FLAGS="-O2 -fno-common"

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then echo "$candidate"; return; fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      GCC4=$(find_input gcc-4.0.4-musl-cxx-v7)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v16)
      MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      GREP=$(find_input grep-2.4-gcc-v10)
      GAWK=$(find_input gawk-3.0.4-gcc-v4)
      GMP=$(find_input gmp-6.2.1-gcc40-v1)
      MPFR=$(find_input mpfr-4.1.0-gcc40-v1)
      MPC=$(find_input mpc-1.2.1-gcc40-v1)
      AUTOCONF=$(find_input autoconf-2.64-gcc-v1)
      PERL=$(find_input perl-5.6.2-configured-gcc-v16)
      M4TOOL=$(find_input m4-1.4.7-gcc-v5)
      BISON=$(find_input bison-3.4.1-gcc-v2)
      FLEX=$(find_input flex-2.6.4-gcc-v1)
      SRC=$(find_input gcc-4.7.4-src)
      AUTOMAKE115_SRC=$(find_input automake-1.15.1-config-sub-src)

      test -x "$GCC4/bin/gcc"
      test -x "$GCC4/bin/g++"
      test -x "$GCC4/libexec/gcc/$TARGET/$SEED_GCC_VERSION/cc1"
      test -x "$GCC4/libexec/gcc/$TARGET/$SEED_GCC_VERSION/cc1plus"
      test -s "$GMP/lib/libgmp.a"
      test -s "$MPFR/lib/libmpfr.a"
      test -s "$MPC/lib/libmpc.a"

      WORK=/tmp/gcc-4.7.4-musl-gcc40
      TOOLBIN="$WORK/tools"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in awk basename cat chmod cmp cp cut date diff dirname echo egrep env expr find grep head ln ls mkdir mv od printf pwd rm rmdir sed sh sleep sort tail tar test touch tr true uname uniq wc; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$AUTOCONF/bin:$PERL/bin:$M4TOOL/bin:$BISON/bin:$FLEX/bin:$GAWK/bin:$GREP/bin:$TOOLBIN:$MAKE/bin:$BINUTILS/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export MAKEINFO=true
      unset MAKEFLAGS MFLAGS MAKELEVEL

      SEED_CC="$TOOLBIN/gcc40"
      $BB cat > "$SEED_CC" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
MANTLE_GCC_LINK_LIBGCC="$GCC4/lib/gcc/$TARGET/$SEED_GCC_VERSION/libgcc.a" \
MANTLE_GCC_LINK_CRT="$MUSL/lib" \
exec "$GCC4/bin/gcc" \
  -B"$GCC4/bin/" \
  -B"$GCC4/lib/gcc/$TARGET/$SEED_GCC_VERSION/" \
  "\$@"
EOF
      $BB chmod 755 "$SEED_CC"
      SEED_CXX="$TOOLBIN/gxx40"
      $BB cat > "$SEED_CXX" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
MANTLE_GCC_LINK_LIBGCC="$GCC4/lib/gcc/$TARGET/$SEED_GCC_VERSION/libgcc.a" \
MANTLE_GCC_LINK_CRT="$MUSL/lib" \
exec "$GCC4/bin/g++" \
  -B"$GCC4/bin/" \
  -B"$GCC4/lib/gcc/$TARGET/$SEED_GCC_VERSION/" \
  "\$@"
EOF
      $BB chmod 755 "$SEED_CXX"

      $BB mkdir -p "$out/bin"
      $BB cat > "$out/bin/as" <<EOF
#!/bin/sh
assembler="\${MANTLE_GCC_ASSEMBLER:-$BINUTILS/bin/as}"
exec "\$assembler" "\$@"
EOF
      $BB chmod 755 "$out/bin/as"
      $BB cat > "$out/bin/ld" <<EOF
#!/bin/sh
libc_archive="\${MANTLE_GCC_LINK_LIBC:-$MUSL/lib/libc.a}"
libm_archive="\${MANTLE_GCC_LINK_LIBM:-$MUSL/lib/libm.a}"
libgcc_archive="\${MANTLE_GCC_LINK_LIBGCC:-$GCC4/lib/gcc/$TARGET/$SEED_GCC_VERSION/libgcc.a}"
zlib_archive="\${MANTLE_GCC_LINK_ZLIB:-$WORK/build/zlib/libz.a}"
crt_dir="\${MANTLE_GCC_LINK_CRT:-$MUSL/lib}"
linker="\${MANTLE_GCC_LINKER:-$BINUTILS/bin/ld}"
link_arguments=
for link_argument in "\$@"; do
  case "\$link_argument" in
    -lgcc|-lgcc_eh) link_arguments="\$link_arguments \$libgcc_archive" ;;
    -lgcc_s) ;;
    crt1.o|crti.o|crtn.o) link_arguments="\$link_arguments \$crt_dir/\$link_argument" ;;
    -lc) link_arguments="\$link_arguments \$libc_archive" ;;
    -lm) link_arguments="\$link_arguments \$libm_archive" ;;
    -lz) link_arguments="\$link_arguments \$zlib_archive" ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
echo "gcc47-final-ld: \$linker \$link_arguments" >&2
exec "\$linker" \$link_arguments
EOF
      $BB chmod 755 "$out/bin/ld"

      $BB cp -R "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"
      export PERL="$PERL/bin/perl"
      export M4="$M4TOOL/bin/m4"
      export BISON_PKGDATADIR="$BISON/share/bison"
      for configure_dir in . fixincludes intl gcc lto-plugin libcpp zlib libstdc++-v3; do
        if test -s "$configure_dir/configure.ac"; then
          (cd "$configure_dir" && "$AUTOCONF/bin/autoconf" --force)
          test -s "$configure_dir/configure"
        fi
      done
      for header_dir in fixincludes intl gcc libcpp; do
        if test -s "$header_dir/configure.ac"; then
          (cd "$header_dir" && "$AUTOCONF/bin/autoheader" --force)
        fi
      done
      M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$BISON/share/bison" \
        "$BISON/bin/bison" -y -o intl/plural.c intl/plural.y
      test -s intl/plural.c
      (cd gcc && "$FLEX/bin/flex" -o gengtype-lex.generated.c gengtype-lex.l)
      $BB cat > gcc/gengtype-lex.c <<'EOF'
#ifdef HOST_GENERATOR_FILE
#include "config.h"
#else
#include "bconfig.h"
#endif
EOF
      $BB cat gcc/gengtype-lex.generated.c >> gcc/gengtype-lex.c
      $BB rm gcc/gengtype-lex.generated.c
      test -s gcc/gengtype-lex.c
      $BB printf '%s\n' \
        configure fixincludes/configure intl/configure intl/plural.c gcc/configure gcc/config.in \
        gcc/gengtype-lex.c lto-plugin/configure libcpp/configure zlib/configure libstdc++-v3/configure \
        > "$WORK/gcc47-regenerated-files.txt"
      for regenerated_file in $($BB cat "$WORK/gcc47-regenerated-files.txt"); do
        test -s "$regenerated_file"
      done
      for config_sub in $($BB find . -name config.sub -type f); do
        $BB cp "$AUTOMAKE115_SRC/lib/config.sub" "$config_sub"
        $BB chmod 755 "$config_sub"
      done
      $BB sed -i '/^case "\${host_os}" in$/a\
  linux-musl*)\
    os_include_dir="os/generic"\
    ;;' libstdc++-v3/configure.host
      $BB grep -q 'linux-musl' libstdc++-v3/configure.host

      $BB mkdir -p "$WORK/build"
      cd "$WORK/build"
      CC="$SEED_CC" \
      CXX="$SEED_CXX" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ranlib" \
      "$WORK/src/configure" \
        --prefix="$out" \
        --libdir="$out/lib" \
        --build="$TARGET" \
        --host="$TARGET" \
        --target="$TARGET" \
        --enable-languages=c,c++ \
        --disable-bootstrap \
        --disable-dependency-tracking \
        --disable-multilib \
        --disable-nls \
        --disable-shared \
        --disable-libssp \
        --disable-libmudflap \
        --disable-libgomp \
        --disable-libquadmath \
        --disable-libatomic \
        --disable-libstdcxx-pch \
        --disable-lto \
        --disable-threads \
        --without-system-zlib \
        --with-sysroot="$MUSL" \
        --with-build-sysroot="$MUSL" \
        --with-native-system-header-dir=/include \
        --with-as="$out/bin/as" \
        --with-ld="$out/bin/ld" \
        --with-gmp="$GMP" \
        --with-mpfr="$MPFR" \
        --with-mpc="$MPC" \
        CPPFLAGS="-I$WORK/src/zlib" \
        CFLAGS="$BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        CXXFLAGS="$BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        CFLAGS_FOR_TARGET="$BUILD_FLAGS -I$MUSL/include" \
        CXXFLAGS_FOR_TARGET="$BUILD_FLAGS -I$MUSL/include" \
        LDFLAGS="-static -L$MUSL/lib -L$GMP/lib -L$MPFR/lib -L$MPC/lib -L$WORK/build/zlib" \
        MAKEINFO=true

      make -j"$JOB_COUNT" all-gcc \
        CC="$SEED_CC" CXX="$SEED_CXX" \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" \
        CFLAGS="$BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        BOOT_CFLAGS="$BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        GMPLIBS="$MPC/lib/libmpc.a $MPFR/lib/libmpfr.a $GMP/lib/libgmp.a" \
        MAKEINFO=true || {
          build_status=$?
          if [ -f "$WORK/build/libcpp/config.log" ]; then
            $BB cat "$WORK/build/libcpp/config.log" >&2
          fi
          exit "$build_status"
        }
      make -j"$JOB_COUNT" all-target-libgcc \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true
      make -j"$JOB_COUNT" all-target-libstdc++-v3 \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true
      make -j"$JOB_COUNT" install-gcc \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true
      make -j"$JOB_COUNT" install-target-libgcc \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true
      make -j"$JOB_COUNT" install-target-libstdc++-v3 \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true

      if [ -x "$out/bin/$TARGET-gcc" ] && [ ! -x "$out/bin/gcc" ]; then
        $BB ln -s "$TARGET-gcc" "$out/bin/gcc"
      fi
      if [ -x "$out/bin/$TARGET-g++" ] && [ ! -x "$out/bin/g++" ]; then
        $BB ln -s "$TARGET-g++" "$out/bin/g++"
      fi
      if [ -x "$out/bin/gcc" ] && [ ! -x "$out/bin/cc" ]; then
        $BB ln -s gcc "$out/bin/cc"
      fi
      if [ -x "$out/bin/g++" ] && [ ! -x "$out/bin/c++" ]; then
        $BB ln -s g++ "$out/bin/c++"
      fi

      test -x "$out/bin/gcc"
      test -x "$out/bin/g++"
      test -x "$out/libexec/gcc/$TARGET/$OUTPUT_GCC_VERSION/cc1"
      test -x "$out/libexec/gcc/$TARGET/$OUTPUT_GCC_VERSION/cc1plus"
      test -s "$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/libgcc.a"
      LIBSTDCXX=$($BB find "$out" -type f -name libstdc++.a | $BB head -n 1)
      LIBSUPCXX=$($BB find "$out" -type f -name libsupc++.a | $BB head -n 1)
      test -n "$LIBSTDCXX"
      test -n "$LIBSUPCXX"
      test -d "$out/include/c++/$OUTPUT_GCC_VERSION"

      # Replace the build-time seed libgcc fallback with the installed GCC 4.7
      # archive while preserving an explicit relocation override seam.
      $BB sed -i \
        -e "s#libgcc_archive=\".*#libgcc_archive=\"\\\${MANTLE_GCC_LINK_LIBGCC:-$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/libgcc.a}\"#" \
        -e 's#zlib_archive=".*#zlib_archive="${MANTLE_GCC_LINK_ZLIB:--lz}"#' \
        "$out/bin/ld"

      $BB cat > "$WORK/positive.c" <<'EOF'
int main(void) { return 0; }
EOF
      MANTLE_GCC_LINK_LIBGCC="$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/libgcc.a" \
        "$out/bin/gcc" -B"$out/bin/" \
          -B"$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/" \
          -I"$MUSL/include" -static "$WORK/positive.c" -o "$WORK/positive-c"
      test -s "$WORK/positive-c"
      $BB chmod 755 "$WORK/positive-c"
      "$WORK/positive-c"

      $BB cat > "$WORK/positive.cpp" <<'EOF'
struct base_value { virtual ~base_value() { } };
struct derived_value : base_value { int value = 42; };
auto add(int left, int right) -> int { return left + right; }
int main() {
  base_value *base = new derived_value();
  derived_value *derived = dynamic_cast<derived_value *>(base);
  int result = 1;
  try {
    if (derived != nullptr) throw add(derived->value, 0);
  } catch (int value) {
    result = value == 42 ? 0 : 1;
  }
  delete base;
  return result;
}
EOF
      MANTLE_GCC_LINK_LIBGCC="$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/libgcc.a" \
        "$out/bin/g++" -std=c++11 -B"$out/bin/" \
          -B"$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/" \
          -I"$MUSL/include" -static "$WORK/positive.cpp" -o "$WORK/positive-cxx"
      test -s "$WORK/positive-cxx"
      $BB chmod 755 "$WORK/positive-cxx"
      "$WORK/positive-cxx"

      $BB printf '%s\n' 'template <typename T> int main(' > "$WORK/rejected.cpp"
      $BB rm -f "$WORK/rejected.o"
      if MANTLE_GCC_LINK_LIBGCC="$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/libgcc.a" \
        "$out/bin/g++" -std=c++11 -B"$out/bin/" \
          -B"$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/" \
          -I"$MUSL/include" -c "$WORK/rejected.cpp" -o "$WORK/rejected.o"; then
        echo "ERROR: GCC 4.7 accepted malformed C++" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      RELOCATED_GCC="$WORK/relocated-gcc47"
      $BB cp -R "$out" "$RELOCATED_GCC"
      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
      MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
      MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
      MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
      MANTLE_GCC_LINK_LIBGCC="$RELOCATED_GCC/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/libgcc.a" \
      MANTLE_GCC_LINK_CRT="$MUSL/lib" \
        "$RELOCATED_GCC/bin/gcc" \
          -B"$RELOCATED_GCC/bin/" \
          -B"$RELOCATED_GCC/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/" \
          -I"$MUSL/include" -static "$WORK/positive.c" \
          -o "$WORK/relocated-positive"
      test -s "$WORK/relocated-positive"
      $BB chmod 755 "$WORK/relocated-positive"
      "$WORK/relocated-positive"

      $BB mkdir -p "$out/share/gcc47-frontier"
      $BB cp "$WORK/gcc47-regenerated-files.txt" "$out/share/gcc47-frontier/regenerated-files.txt"
      $BB printf '%s\n' \
        'NON_ADMISSION: component regeneration and runtime pass; normalized-provider admission remains separate' \
        > "$out/share/gcc47-frontier/NON_ADMISSION"
      echo "GCC 4.7.4 C/C++11 runtime, relocation, and rejection checks passed; not provider admission"
    "%
  ],
  inputs = [
    crunch.derivationFile "gcc-4.0-musl-cxx.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "gmp-6.2.1.ncl",
    crunch.derivationFile "mpfr-4.1.0.ncl",
    crunch.derivationFile "mpc-1.2.1.ncl",
    crunch.derivationFile "autoconf-2.64-gcc.ncl",
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "bison-3.4.1-gcc.ncl",
    crunch.derivationFile "flex-2.6.4-gcc.ncl",
    gcc47_src,
    automake115_src,
  ],
} | crunch.Derivation
