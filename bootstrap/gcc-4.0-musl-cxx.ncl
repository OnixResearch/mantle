# Regenerated GCC 4.0.4 C/C++ early-native parity output.
#
# GCC pass6, musl pass5, and clean binutils provide the same-version predecessor
# closure. This stage requires real cc1/cc1plus, libgcc, libstdc++, and
# libsupc++ outputs and executes C plus exceptions/RTTI/new/delete/dynamic_cast
# runtime smokes. The row claim is bounded to the receipted behavior matrix; it
# does not establish general compiler correctness or provider/seed admission.

let crunch = import "lib.ncl" in

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
  name = "gcc-4.0.4-musl-cxx-v8",
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

      SEED_GCC=$(find_input gcc-4.0.4-musl-pass6-v1)
      MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v17)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      BISON=$(find_input bison-3.4.1-gcc-v2)
      FLEX=$(find_input flex-2.6.4-gcc-v1)
      M4TOOL=$(find_input m4-1.4.7-gcc-v5)
      AUTOCONF213=$(find_input autoconf-2.13-gcc-v1)
      AUTOCONF259=$(find_input autoconf-2.59-gcc-v2)
      PERL=$(find_input perl-5.6.2-configured-gcc-v16)
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

      WORK=/tmp/gcc-4.0.4-musl-cxx
      TOOLBIN="$WORK/tools"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in awk basename cat chmod cmp cp cut dirname echo egrep expr find grep head ln ls mkdir mv od printf pwd rm rmdir sed sh sort tail tar test touch tr true uname uniq wc; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      # The admitted row must not inherit ambient command discovery. Every
      # executable reachable by name is supplied by a declared derivation or
      # by the bounded BusyBox applet directory above.
      export PATH="$TOOLBIN:$BINUTILS/bin:$MAKE/bin:$BISON/bin:$FLEX/bin:$M4TOOL/bin:$AUTOCONF213/bin:$AUTOCONF259/bin:$PERL/bin:$STAGE0/bin"
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
      test -f gcc/gengtype-yacc.y || { echo "ERROR: gengtype grammar missing" >&2; exit 1; }
      test -f gcc/gengtype-lex.l || { echo "ERROR: gengtype scanner missing" >&2; exit 1; }

      RANDOM_SEED_SOURCE=gcc/toplev.c
      RANDOM_SEED_OLD='      value = local_tick ^ getpid ();'
      RANDOM_SEED_NEW='      value = crc32_string (0, main_input_filename ? main_input_filename : "mantle-gcc40-no-input");'
      RANDOM_SEED_ASSIGN_OLD='      flag_random_seed = random_seed;'
      RANDOM_SEED_ASSIGN_NEW='      flag_random_seed = random_seed; local_tick = -1;'
      test "$($BB grep -Fxc "$RANDOM_SEED_OLD" "$RANDOM_SEED_SOURCE")" -eq 1
      test "$($BB grep -Fxc "$RANDOM_SEED_ASSIGN_OLD" "$RANDOM_SEED_SOURCE")" -eq 1
      $BB sed -i "s|$RANDOM_SEED_OLD|$RANDOM_SEED_NEW|" "$RANDOM_SEED_SOURCE"
      $BB sed -i "s|$RANDOM_SEED_ASSIGN_OLD|$RANDOM_SEED_ASSIGN_NEW|" "$RANDOM_SEED_SOURCE"
      test "$($BB grep -Fxc "$RANDOM_SEED_NEW" "$RANDOM_SEED_SOURCE")" -eq 1
      test "$($BB grep -Fxc "$RANDOM_SEED_ASSIGN_NEW" "$RANDOM_SEED_SOURCE")" -eq 1
      test "$($BB grep -Fxc "$RANDOM_SEED_OLD" "$RANDOM_SEED_SOURCE")" -eq 0

      M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$BISON/share/bison" \
        "$BISON/bin/bison" -d -o gcc/gengtype-yacc.c gcc/gengtype-yacc.y
      test -s gcc/gengtype-yacc.c
      test -s gcc/gengtype-yacc.h
      "$FLEX/bin/flex" -o gcc/gengtype-lex.c gcc/gengtype-lex.l
      test -s gcc/gengtype-lex.c
      # GCC 4.0.4 documents exact Autoconf versions for both regeneration
      # boundaries: 2.13 for top-level configure.in and 2.59 for gcc/.
      AC_MACRODIR="$AUTOCONF213/share/autoconf" \
      M4="$M4TOOL/bin/m4" \
      AWK="$TOOLBIN/awk" \
        /bin/sh "$AUTOCONF213/bin/autoconf" configure.in > configure.regenerated
      test -s configure.regenerated
      $BB grep -q 'autoconf version 2.13' configure.regenerated
      $BB mv configure.regenerated configure
      $BB chmod 755 configure
      (
        cd gcc
        /bin/sh "$AUTOCONF259/bin/autoconf"
      )
      test -s gcc/configure

      for config_sub in $($BB find . -name config.sub -type f); do
        $BB cp "$AUTOMAKE115_SRC/lib/config.sub" "$config_sub"
        $BB chmod 755 "$config_sub"
      done
      $BB sed -i '/^case "${host_os}" in$/a\
  linux-musl*)\
    os_include_dir="os/generic"\
    ;;' libstdc++-v3/configure.host
      $BB grep -q 'linux-musl' libstdc++-v3/configure.host
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
        --enable-languages=c,c++ \
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
      # Bison 3 removed the old skeleton-provided YYLEX compatibility macro,
      # while GCC 4.0's semantic actions still use it to force lookahead.
      # Restore exactly the historical no-argument call at the grammar edge.
      GRAMMAR_PROLOGUE=$($BB printf '\045\173')
      $BB sed -i "/^$GRAMMAR_PROLOGUE$/a#define YYLEX yylex ()" c-parse.y
      M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$BISON/share/bison" \
        "$BISON/bin/bison" -o c-parse.c c-parse.y
      test -s c-parse.c
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
      make -j"$JOB_COUNT" all-target-libstdc++-v3 \
        CC="$SEED_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ranlib" \
        MAKEINFO=true || {
          build_status=$?
          if [ -f "$WORK/build/$TARGET/libstdc++-v3/config.log" ]; then
            $BB cat "$WORK/build/$TARGET/libstdc++-v3/config.log" >&2
          fi
          exit "$build_status"
        }
      make -j"$JOB_COUNT" install-gcc \
        CC="$SEED_CC" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ranlib" \
        MAKEINFO=true
      make -j"$JOB_COUNT" install-target-libstdc++-v3 \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ranlib" \
        MAKEINFO=true

      GENERATED_SOURCE_DIR="$out/share/mantle-bootstrap/generated-gcc40"
      GENERATOR_DIR="$out/libexec/gcc/$TARGET/$GCC_VERSION/generators"
      LIBIBERTY_BUILD="$WORK/build/build-$TARGET/libiberty/libiberty.a"
      $BB mkdir -p "$GENERATED_SOURCE_DIR" "$GENERATOR_DIR" "$out/lib"
      test -s "$WORK/build/gcc/c-parse.c"
      test -s "$WORK/src/gcc/gengtype-yacc.c"
      test -s "$WORK/src/gcc/gengtype-lex.c"
      test -s "$WORK/src/configure"
      test -s "$WORK/src/gcc/configure"
      $BB cp "$WORK/build/gcc/c-parse.c" "$GENERATED_SOURCE_DIR/c-parse.c"
      $BB cp "$WORK/src/gcc/gengtype-yacc.c" "$GENERATED_SOURCE_DIR/gengtype-yacc.c"
      $BB cp "$WORK/src/gcc/gengtype-lex.c" "$GENERATED_SOURCE_DIR/gengtype-lex.c"
      $BB cp "$WORK/src/configure" "$GENERATED_SOURCE_DIR/configure-autoconf-2.13"
      $BB cp "$WORK/src/gcc/configure" "$GENERATED_SOURCE_DIR/gcc-configure-autoconf-2.59"
      $BB printf '%s\n' \
        'default_random_seed=main-input-filename-crc32' \
        'default_random_seed_local_tick=disabled' \
        > "$GENERATED_SOURCE_DIR/random-seed-determinism.txt"
      for generator in genattrtab genoutput genemit genrecog genextract gengtype; do
        test -x "$WORK/build/gcc/build/$generator"
        $BB cp "$WORK/build/gcc/build/$generator" "$GENERATOR_DIR/$generator"
        $BB chmod 755 "$GENERATOR_DIR/$generator"
      done
      test -s "$LIBIBERTY_BUILD"
      $BB cp "$LIBIBERTY_BUILD" "$out/lib/libiberty.a"

      if [ -x "$out/bin/$TARGET-g++" ] && [ ! -x "$out/bin/g++" ]; then
        $BB ln -s "$TARGET-g++" "$out/bin/g++"
      fi
      if [ -x "$out/bin/g++" ] && [ ! -x "$out/bin/c++" ]; then
        $BB ln -s g++ "$out/bin/c++"
      fi

      echo 'gcc40-cxx-check: installed-binaries' >&2
      test -x "$out/bin/gcc"
      test -x "$out/bin/g++"
      test -x "$out/bin/c++"
      test -x "$out/bin/as"
      test -x "$out/bin/ld"
      test -x "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -x "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1plus"
      test -s "$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"

      CC1PLUS="$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1plus"
      $BB cat > "$WORK/random-seed-probe.cc" <<'EOF'
namespace { struct state { static int value; }; int state::value = 42; }
int read_value() { return state::value; }
EOF
      "$CC1PLUS" -quiet -O2 "$WORK/random-seed-probe.cc" -o "$WORK/random-seed-default-a.s"
      "$CC1PLUS" -quiet -O2 "$WORK/random-seed-probe.cc" -o "$WORK/random-seed-default-b.s"
      $BB cmp "$WORK/random-seed-default-a.s" "$WORK/random-seed-default-b.s"
      "$CC1PLUS" -quiet -O2 -frandom-seed=mantle-negative-a \
        "$WORK/random-seed-probe.cc" -o "$WORK/random-seed-explicit-a.s"
      "$CC1PLUS" -quiet -O2 -frandom-seed=mantle-negative-b \
        "$WORK/random-seed-probe.cc" -o "$WORK/random-seed-explicit-b.s"
      if $BB cmp "$WORK/random-seed-explicit-a.s" "$WORK/random-seed-explicit-b.s"; then
        echo "ERROR: GCC random-seed negative control produced equal assembly" >&2
        exit 1
      fi

      LIBSTDCXX=$($BB find "$out" -type f -name libstdc++.a | $BB head -n 1)
      LIBSUPCXX=$($BB find "$out" -type f -name libsupc++.a | $BB head -n 1)
      test -n "$LIBSTDCXX"
      test -n "$LIBSUPCXX"
      test -d "$out/include/c++/$GCC_VERSION"
      "$BINUTILS/bin/ar" t "$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" > "$WORK/libgcc-members.txt"
      "$BINUTILS/bin/ar" t "$LIBSTDCXX" > "$WORK/libstdcxx-members.txt"
      "$BINUTILS/bin/ar" t "$LIBSUPCXX" > "$WORK/libsupcxx-members.txt"
      test -s "$WORK/libgcc-members.txt"
      test -s "$WORK/libstdcxx-members.txt"
      test -s "$WORK/libsupcxx-members.txt"

      $BB cat > "$out/bin/ld" <<EOF
#!/bin/sh
libc_archive="\${MANTLE_GCC_LINK_LIBC:-$MUSL/lib/libc.a}"
libm_archive="\${MANTLE_GCC_LINK_LIBM:-$MUSL/lib/libm.a}"
libgcc_archive="\${MANTLE_GCC_LINK_LIBGCC:-$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a}"
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
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
echo "gcc40-cxx-final-ld: \$linker \$link_arguments" >&2
exec "\$linker" \$link_arguments
EOF
      $BB chmod 755 "$out/bin/ld"

      "$out/bin/gcc" -v > "$WORK/gcc-version.out" 2> "$WORK/gcc-version.err"
      "$out/bin/g++" -v > "$WORK/gxx-version.out" 2> "$WORK/gxx-version.err"
      "$out/bin/gcc" -dumpspecs > "$WORK/specs.txt"
      test -s "$WORK/specs.txt"

      $BB cat > "$WORK/c-runtime.c" <<'EOF'
#define MAX_UNSIGNED_WORD 18446744073709551615ULL
#define ROUNDED_MAX_FLOAT 18446744073709551616.0f
int main(void)
{
  volatile unsigned long long value = MAX_UNSIGNED_WORD;
  float converted = (float)value;
  return converted == ROUNDED_MAX_FLOAT ? 0 : 1;
}
EOF
      "$out/bin/gcc" -I"$MUSL/include" -static "$WORK/c-runtime.c" -o "$WORK/c-runtime"
      test -s "$WORK/c-runtime"
      $BB chmod 755 "$WORK/c-runtime"
      "$WORK/c-runtime"

      $BB cat > "$WORK/cxx-runtime.cpp" <<'EOF'
struct base_value {
  virtual ~base_value() { }
};
struct derived_value : base_value {
  int value;
  derived_value() : value(42) { }
};
int main()
{
  base_value *base = new derived_value();
  derived_value *derived = dynamic_cast<derived_value *>(base);
  int result = 1;
  try {
    if (derived != 0) throw derived->value;
  } catch (int value) {
    result = value == 42 ? 0 : 1;
  }
  delete base;
  return result;
}
EOF
      "$out/bin/g++" -I"$MUSL/include" -static \
        "$WORK/cxx-runtime.cpp" -o "$WORK/cxx-runtime"
      test -s "$WORK/cxx-runtime"
      $BB chmod 755 "$WORK/cxx-runtime"
      "$WORK/cxx-runtime"
      "$BINUTILS/bin/readelf" -h "$WORK/cxx-runtime" > "$WORK/cxx-runtime-header.txt"
      $BB grep -q 'ELF' "$WORK/cxx-runtime-header.txt"

      $BB cat > "$WORK/demangle-runtime.c" <<'EOF'
#include <demangle.h>
#include <stdlib.h>
#include <string.h>
int main(void)
{
  const int options = DMGL_PARAMS | DMGL_ANSI | DMGL_TYPES;
  char *accepted = cplus_demangle("_ZN3foo3barEs", options);
  char *rejected = cplus_demangle("_ZN3foo3barE!", options);
  int status = 1;
  if (accepted != 0 && rejected == 0 && strcmp(accepted, "foo::bar(short)") == 0)
    status = 0;
  free(accepted);
  free(rejected);
  return status;
}
EOF
      "$out/bin/gcc" -I"$MUSL/include" -I"$WORK/src/include" -static \
        "$WORK/demangle-runtime.c" "$out/lib/libiberty.a" \
        -o "$WORK/demangle-runtime"
      test -s "$WORK/demangle-runtime"
      $BB chmod 755 "$WORK/demangle-runtime"
      "$WORK/demangle-runtime"

      # Prove the installed compiler can execute after its output tree moves
      # away from the sandbox's logical $out mount.  All absolute tool/runtime
      # references cross explicit override seams; GCC's own executables and
      # archives are selected from the relocated tree.
      RELOCATED_GCC="$WORK/relocated-gcc"
      $BB cp -R "$out" "$RELOCATED_GCC"
      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
      MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
      MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
      MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
      MANTLE_GCC_LINK_LIBGCC="$RELOCATED_GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
      MANTLE_GCC_LINK_CRT="$MUSL/lib" \
        "$RELOCATED_GCC/bin/gcc" \
          -B"$RELOCATED_GCC/bin/" \
          -B"$RELOCATED_GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
          -I"$MUSL/include" -static "$WORK/c-runtime.c" \
          -o "$WORK/relocated-c-runtime"
      test -s "$WORK/relocated-c-runtime"
      $BB chmod 755 "$WORK/relocated-c-runtime"
      "$WORK/relocated-c-runtime"

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected-c.o"
      if "$out/bin/gcc" -I"$MUSL/include" -c "$WORK/rejected.c" -o "$WORK/rejected-c.o"; then
        echo "ERROR: GCC C++ stage accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-c.o"

      $BB printf '%s\n' 'template <typename T> int main(' > "$WORK/rejected.cpp"
      $BB rm -f "$WORK/rejected-cxx.o"
      if "$out/bin/g++" -I"$MUSL/include" -c "$WORK/rejected.cpp" -o "$WORK/rejected-cxx.o"; then
        echo "ERROR: GCC C++ stage accepted malformed C++" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-cxx.o"

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/early-native-gcc40-row.txt" <<'EOF'
claim: bounded early-native GCC 4.0 row behavior matrix passed
artifacts: gcc,g++,cc1,cc1plus,libgcc.a,libstdc++.a,libsupc++.a,generators,demangler
positive: C static runtime; C++ exceptions,RTTI,new,delete,dynamic_cast; relocated C runtime
negative: malformed C rejected without output; malformed C++ rejected without output
non_claims: general compiler correctness; provider admission; seed admission
EOF
      echo "GCC 4.0.4 regenerated C/C++ early-native row behavior matrix passed"
    "%
  ],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "gcc-4.0-musl-pass6.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "bison-3.4.1-gcc.ncl",
    crunch.derivationFile "flex-2.6.4-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "autoconf-2.13-gcc.ncl",
    crunch.derivationFile "autoconf-2.59-gcc.ncl",
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
    gcc_src,
    automake115_src,
  ],
} | crunch.Derivation
