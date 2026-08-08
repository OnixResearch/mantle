# Final GCC 10.5.0 cycle rebuild over musl 1.2.5.
#
# This closes the bounded compiler/libc runtime cycle and requires static and
# shared libgcc/libstdc++ behavior without claiming general correctness.

let crunch = import "lib.ncl" in
let gcc10_v1 = crunch.derivationFile "gcc-10.ncl" in
let musl125 = crunch.derivationFile "musl-full.ncl" in

let gcc10_src = crunch.derivationFile "gcc-10.5.0-regenerated-source.ncl" in

{
  name = "gcc-10.5.0-musl-final-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=10.5.0
      JOB_COUNT=1
      BUILD_FLAGS='-O2 -fno-common'
      WORK=/tmp/gcc-10.5.0-musl-final
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

      GCC_SEED=$(find_input gcc-10.5.0-musl-gcc47-v2)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v17)
      MUSL=$(find_input musl-1.2.5-gcc10-v1)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      GREP=$(find_input grep-2.4-gcc-v10)
      GAWK=$(find_input gawk-3.0.4-gcc-v4)
      M4=$(find_input m4-1.4.7-gcc-v5)
      FLEX=$(find_input flex-2.6.4-gcc-v1)
      BISON=$(find_input bison-3.4.1-gcc-v2)
      GMP=$(find_input gmp-6.2.1-gcc40-v1)
      MPFR=$(find_input mpfr-4.1.0-gcc40-v1)
      MPC=$(find_input mpc-1.2.1-gcc40-v1)
      SRC=$(find_input gcc-10.5.0-regenerated-source-v1)

      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN" "$out/bin"
      for applet in basename cat chmod cmp cp cut date diff dirname echo egrep env expr false find grep head install ln ls mkdir mv od printf pwd readlink rm rmdir sed sh sleep sort tail tar test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
      $BB ln -sf "$GAWK/bin/awk" "$TOOLBIN/awk"
      $BB ln -sf "$GAWK/bin/awk" "$TOOLBIN/gawk"
      $BB ln -sf "$M4/bin/m4" "$TOOLBIN/m4"
      $BB ln -sf "$FLEX/bin/flex" "$TOOLBIN/flex"
      $BB ln -sf "$BISON/bin/bison" "$TOOLBIN/bison"
      $BB ln -sf "$MAKE/bin/make" "$TOOLBIN/make"
      $BB cat > "$TOOLBIN/ldconfig" <<'EOF'
#!/bin/sh
exit 0
EOF
      $BB chmod 755 "$TOOLBIN/ldconfig"
      export PATH="$TOOLBIN:$BINUTILS/bin:$STAGE0/bin"
      export M4="$M4/bin/m4"
      export BISON_PKGDATADIR="$BISON/share/bison"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      unset MAKEFLAGS MFLAGS MAKELEVEL

      PERCENT=$($BB printf '\045')
      $BB cat > "$TOOLBIN/musl125.specs" <<EOF
*startfile:
${PERCENT}{!nostdlib:${PERCENT}{!shared:$MUSL/lib/crt1.o} $MUSL/lib/crti.o ${PERCENT}{static:$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/crtbeginT.o;shared:$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/crtbeginS.o;:$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/crtbegin.o}}

*endfile:
${PERCENT}{!nostdlib:${PERCENT}{shared:$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/crtendS.o;:$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/crtend.o} $MUSL/lib/crtn.o}
EOF

      $BB cat > "$TOOLBIN/gcc-seed" <<EOF
#!/bin/sh
export MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as"
export MANTLE_GCC_LINKER="$BINUTILS/bin/ld"
export MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a"
export MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a"
export MANTLE_GCC_LINK_LIBGCC="$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
export MANTLE_GCC_LINK_CRT="$MUSL/lib"
exec "$GCC_SEED/bin/gcc" -specs="$TOOLBIN/musl125.specs" -B"$GCC_SEED/bin/" -B"$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/" -isystem "$MUSL/include" "\$@"
EOF
      $BB cat > "$TOOLBIN/gxx-seed" <<EOF
#!/bin/sh
export MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as"
export MANTLE_GCC_LINKER="$BINUTILS/bin/ld"
export MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a"
export MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a"
export MANTLE_GCC_LINK_LIBGCC="$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
export MANTLE_GCC_LINK_CRT="$MUSL/lib"
exec "$GCC_SEED/bin/g++" -specs="$TOOLBIN/musl125.specs" -B"$GCC_SEED/bin/" -B"$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/" \
  -nostdinc++ \
  -isystem "$GCC_SEED/include/c++/$GCC_VERSION" \
  -isystem "$GCC_SEED/include/c++/$GCC_VERSION/$TARGET" \
  -isystem "$GCC_SEED/include/c++/$GCC_VERSION/backward" \
  -isystem "$MUSL/include" "\$@"
EOF
      $BB chmod 755 "$TOOLBIN/gcc-seed" "$TOOLBIN/gxx-seed"
      HOST_CC="$TOOLBIN/gcc-seed"
      HOST_CXX="$TOOLBIN/gxx-seed"

      $BB cat > "$out/bin/as" <<EOF
#!/bin/sh
exec "\${MANTLE_GCC_ASSEMBLER:-$BINUTILS/bin/as}" "\$@"
EOF
      $BB chmod 755 "$out/bin/as"

      $BB cat > "$out/bin/ld" <<EOF
#!/bin/sh
link_mode=dynamic
for mode_argument in "\$@"; do
  case "\$mode_argument" in
    -static) link_mode=static ;;
    -shared) link_mode=shared ;;
  esac
done
dynamic_library_argument() {
  dynamic_library_path=\$1
  case "\$dynamic_library_path" in
    /*)
      dynamic_library_dir=\${dynamic_library_path%/*}
      dynamic_library_name=\${dynamic_library_path##*/}
      printf '%s' "-L\$dynamic_library_dir -l:\$dynamic_library_name"
      ;;
    *) printf '%s' "\$dynamic_library_path" ;;
  esac
}
if [ "\$link_mode" = static ]; then
  libc_library="\${MANTLE_GCC_LINK_LIBC:-$MUSL/lib/libc.a}"
  libm_library="\${MANTLE_GCC_LINK_LIBM:-$MUSL/lib/libm.a}"
else
  libc_library=\$(dynamic_library_argument "\${MANTLE_GCC_LINK_LIBC:-$MUSL/lib/libc.so}")
  libm_library=\$(dynamic_library_argument "\${MANTLE_GCC_LINK_LIBM:-$MUSL/lib/libc.so}")
fi
libgcc_archive="\${MANTLE_GCC_LINK_LIBGCC:-$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a}"
libgcc_eh_archive="\${MANTLE_GCC_LINK_LIBGCC_EH:-$GCC_SEED/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a}"
libgcc_shared="\${MANTLE_GCC_LINK_LIBGCC_S:-$WORK/build/$TARGET/libgcc/libgcc_s.so.1}"
zlib_archive="\${MANTLE_GCC_LINK_ZLIB:-$WORK/build/zlib/libz.a}"
crt_dir="\${MANTLE_GCC_LINK_CRT:-$MUSL/lib}"
linker="\${MANTLE_GCC_LINKER:-$BINUTILS/bin/ld}"
link_arguments=
for link_argument in "\$@"; do
  case "\$link_argument" in
    -lgcc) link_arguments="\$link_arguments \$libgcc_archive" ;;
    -lgcc_eh) link_arguments="\$link_arguments \$libgcc_eh_archive" ;;
    -lgcc_s) link_arguments="\$link_arguments \$libgcc_shared" ;;
    crt1.o|Scrt1.o|rcrt1.o|crti.o|crtn.o) link_arguments="\$link_arguments \$crt_dir/\$link_argument" ;;
    -lc) link_arguments="\$link_arguments \$libc_library" ;;
    -lm) link_arguments="\$link_arguments \$libm_library" ;;
    -lpthread|-ldl|-lrt|-lutil) link_arguments="\$link_arguments \$libc_library" ;;
    -lz) link_arguments="\$link_arguments \$zlib_archive" ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
echo "gcc10-final-cycle-ld: \$linker \$link_arguments" >&2
exec "\$linker" \$link_arguments
EOF
      $BB chmod 755 "$out/bin/ld"

      $BB cp -R "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      test -s "$WORK/src/.mantle-regeneration/gcc-10.5.0-files.txt"
      for regenerated_file in $($BB cat "$WORK/src/.mantle-regeneration/gcc-10.5.0-files.txt"); do
        test -s "$WORK/src/$regenerated_file"
        $BB touch "$WORK/src/$regenerated_file"
      done
      # Shared libgcc's include iterator can contain duplicate source names.
      # filter-out removes every duplicate at once while the include list still
      # has one entry per original item, so later inclusions can see an empty
      # source. Suppress rules only after the iterator is exhausted.
      SHARED_OBJECT_GUARD_COUNT_EXPECTED=1
      shared_object_rules="$WORK/src/libgcc/shared-object.mk"
      $BB grep -F -q '$(error Unsupported file type: $o)' "$shared_object_rules"
      $BB sed -i '/^base := /i ifneq ($(strip $(o)),)' "$shared_object_rules"
      $BB printf '\nendif\n' >> "$shared_object_rules"
      shared_object_guard_count=$($BB grep -F -c 'ifneq ($(strip $(o)),)' "$shared_object_rules")
      test "$shared_object_guard_count" = "$SHARED_OBJECT_GUARD_COUNT_EXPECTED"
      dynamic_linker_sources=$($BB grep -rl '/lib/ld-musl-x86_64.so.1' "$WORK/src/gcc/config")
      test -n "$dynamic_linker_sources"
      for dynamic_linker_source in $dynamic_linker_sources; do
        $BB sed -i \
          "s#/lib/ld-musl-x86_64.so.1#$MUSL/lib/ld-musl-x86_64.so.1#g" \
          "$dynamic_linker_source"
      done
      $BB grep -rl "$MUSL/lib/ld-musl-x86_64.so.1" "$WORK/src/gcc/config" > "$WORK/dynamic-linker-sources.txt"
      test -s "$WORK/dynamic-linker-sources.txt"
      $BB mkdir -p "$WORK/build"
      cd "$WORK/build"

      CC="$HOST_CC" \
      CXX="$HOST_CXX" \
      CPP="$HOST_CC -E" \
      CXXCPP="$HOST_CXX -E" \
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
        --enable-shared \
        --enable-static \
        --enable-threads=posix \
        --disable-libssp \
        --disable-libgomp \
        --disable-libquadmath \
        --disable-libatomic \
        --disable-libstdcxx-pch \
        --disable-libsanitizer \
        --disable-libvtv \
        --disable-libmpx \
        --disable-libcc1 \
        --disable-lto \
        --disable-plugin \
        --disable-default-pie \
        --without-isl \
        --without-zstd \
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
        CFLAGS="-std=gnu11 $BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        CXXFLAGS="-std=gnu++11 $BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        CFLAGS_FOR_TARGET="$BUILD_FLAGS -I$MUSL/include" \
        CXXFLAGS_FOR_TARGET="$BUILD_FLAGS -I$MUSL/include" \
        LDFLAGS="-static -L$MUSL/lib -L$GMP/lib -L$MPFR/lib -L$MPC/lib -L$WORK/build/zlib" \
        MAKEINFO=true || {
          configure_status=$?
          $BB cat "$WORK/build/config.log" >&2
          exit "$configure_status"
        }

      make -j"$JOB_COUNT" all-gcc \
        CC="$HOST_CC" CXX="$HOST_CXX" \
        CPP="$HOST_CC -E" CXXCPP="$HOST_CXX -E" \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" \
        CFLAGS="-std=gnu11 $BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        CXXFLAGS="-std=gnu++11 $BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        BOOT_CFLAGS="$BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        GMPLIBS="$MPC/lib/libmpc.a $MPFR/lib/libmpfr.a $GMP/lib/libgmp.a" \
        MAKEINFO=true || {
          build_status=$?
          for failure_log in "$WORK/build/intl/config.log" "$WORK/build/gcc/config.log"; do
            if [ -f "$failure_log" ]; then
              $BB cat "$failure_log" >&2
            fi
          done
          exit "$build_status"
        }
      make -j"$JOB_COUNT" all-target-libgcc \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true
      test -s "$WORK/build/$TARGET/libgcc/libgcc_s.so.1"
      make -j"$JOB_COUNT" configure-target-libstdc++-v3 \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true
      LIBSTDCXX_LIBTOOL="$WORK/build/$TARGET/libstdc++-v3/libtool"
      test -f "$LIBSTDCXX_LIBTOOL"
      $BB grep -q '^predep_objects=' "$LIBSTDCXX_LIBTOOL"
      $BB grep -q '^postdep_objects=' "$LIBSTDCXX_LIBTOOL"
      $BB sed -i \
        -e "s#^predep_objects=.*#predep_objects='$WORK/build/gcc/crtbeginS.o'#" \
        -e "s#^postdep_objects=.*#postdep_objects='$WORK/build/gcc/crtendS.o'#" \
        "$LIBSTDCXX_LIBTOOL"
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
      test -x "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -x "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1plus"
      test -s "$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
      LIBGCC_S=$($BB find "$out" -type f -name libgcc_s.so.1 | $BB head -n 1)
      LIBGCC_EH=$($BB find "$out" -type f -name libgcc_eh.a | $BB head -n 1)
      LIBSTDCXX_A=$($BB find "$out" -type f -name libstdc++.a | $BB head -n 1)
      LIBSTDCXX_SO=$($BB find "$out" -type f -name 'libstdc++.so.6.0.28' | $BB head -n 1)
      LIBSUPCXX=$($BB find "$out" -type f -name libsupc++.a | $BB head -n 1)
      test -n "$LIBGCC_S"
      test -n "$LIBGCC_EH"
      test -n "$LIBSTDCXX_A"
      test -n "$LIBSTDCXX_SO"
      test -n "$LIBSUPCXX"
      test -s "$LIBGCC_S"
      test -s "$LIBSTDCXX_SO"
      "$BINUTILS/bin/readelf" -h "$LIBGCC_S" | "$GREP/bin/grep" -q 'DYN'
      "$BINUTILS/bin/readelf" -h "$LIBSTDCXX_SO" | "$GREP/bin/grep" -q 'DYN'
      for shared_runtime in "$LIBGCC_S" "$LIBSTDCXX_SO"; do
        "$BINUTILS/bin/readelf" -d "$shared_runtime" | "$GREP/bin/grep" -q 'NEEDED.*\[libc.so\]'
        if "$BINUTILS/bin/readelf" -d "$shared_runtime" | "$GREP/bin/grep" -q 'NEEDED.*\[/'; then
          echo "ERROR: final GCC shared runtime retains an absolute libc dependency: $shared_runtime" >&2
          exit 1
        fi
      done

      $BB sed -i \
        -e "s#libgcc_archive=\".*#libgcc_archive=\"\\\${MANTLE_GCC_LINK_LIBGCC:-$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a}\"#" \
        -e "s#libgcc_eh_archive=\".*#libgcc_eh_archive=\"\\\${MANTLE_GCC_LINK_LIBGCC_EH:-$LIBGCC_EH}\"#" \
        -e "s#libgcc_shared=\".*#libgcc_shared=\"\\\${MANTLE_GCC_LINK_LIBGCC_S:-$LIBGCC_S}\"#" \
        -e 's#zlib_archive=".*#zlib_archive="${MANTLE_GCC_LINK_ZLIB:--lz}"#' \
        "$out/bin/ld"

      $BB cat > "$WORK/positive.cpp" <<'EOF'
struct base_value { virtual ~base_value() { } };
struct derived_value final : base_value { int value = 42; };
constexpr int identity(int value) { return value; }
int main() {
  base_value *base = new derived_value();
  derived_value *derived = dynamic_cast<derived_value *>(base);
  int result = 1;
  try {
    if (derived != nullptr) throw identity(derived->value);
  } catch (int value) {
    result = value == 42 ? 0 : 1;
  }
  delete base;
  return result;
}
EOF

      MANTLE_GCC_LINK_LIBGCC="$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
        "$out/bin/g++" -std=c++17 -B"$out/bin/" \
          -B"$out/lib/gcc/$TARGET/$GCC_VERSION/" \
          -I"$MUSL/include" -static "$WORK/positive.cpp" -o "$WORK/positive-static"
      test -s "$WORK/positive-static"
      $BB chmod 755 "$WORK/positive-static"
      "$WORK/positive-static"

      LIBSTDCXX_DIR=${LIBSTDCXX_SO%/*}
      LIBGCC_S_DIR=${LIBGCC_S%/*}
      MANTLE_GCC_LINK_LIBGCC="$out/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
      MANTLE_GCC_LINK_LIBGCC_S="$LIBGCC_S" \
        "$out/bin/g++" -std=c++17 -B"$out/bin/" \
          -B"$out/lib/gcc/$TARGET/$GCC_VERSION/" \
          -I"$MUSL/include" \
          -Wl,-dynamic-linker,"$MUSL/lib/ld-musl-x86_64.so.1" \
          -Wl,-rpath,"$LIBSTDCXX_DIR" -Wl,-rpath,"$LIBGCC_S_DIR" \
          "$WORK/positive.cpp" -o "$WORK/positive-dynamic"
      test -s "$WORK/positive-dynamic"
      $BB chmod 755 "$WORK/positive-dynamic"
      "$WORK/positive-dynamic"
      "$BINUTILS/bin/readelf" -d "$WORK/positive-dynamic" | "$GREP/bin/grep" -q 'libstdc++.so.6'
      "$BINUTILS/bin/readelf" -d "$WORK/positive-dynamic" | "$GREP/bin/grep" -q 'libgcc_s.so.1'

      $BB printf '%s\n' 'template <typename T> int main(' > "$WORK/rejected.cpp"
      $BB rm -f "$WORK/rejected.o"
      if "$out/bin/g++" -std=c++17 -B"$out/bin/" \
        -B"$out/lib/gcc/$TARGET/$GCC_VERSION/" \
        -I"$MUSL/include" -c "$WORK/rejected.cpp" -o "$WORK/rejected.o"; then
        echo "ERROR: final GCC 10 accepted malformed C++" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      RELOCATED_GCC="$WORK/relocated-gcc10"
      $BB cp -R "$out" "$RELOCATED_GCC"
      RELOCATED_LIBGCC_S=$($BB find "$RELOCATED_GCC" -type f -name libgcc_s.so.1 | $BB head -n 1)
      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
      MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
      MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
      MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
      MANTLE_GCC_LINK_LIBGCC="$RELOCATED_GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
      MANTLE_GCC_LINK_LIBGCC_S="$RELOCATED_LIBGCC_S" \
      MANTLE_GCC_LINK_LIBGCC_EH="$RELOCATED_GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc_eh.a" \
      MANTLE_GCC_LINK_CRT="$MUSL/lib" \
        "$RELOCATED_GCC/bin/g++" -std=c++17 -static \
          -B"$RELOCATED_GCC/bin/" \
          -B"$RELOCATED_GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
          -I"$MUSL/include" "$WORK/positive.cpp" \
          -o "$WORK/relocated-positive"
      test -s "$WORK/relocated-positive"
      $BB chmod 755 "$WORK/relocated-positive"
      "$WORK/relocated-positive"

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cp "$WORK/src/.mantle-regeneration/gcc-10.5.0-files.txt" "$out/share/mantle-bootstrap/gcc10-final-regenerated-files.txt"
      $BB cat > "$out/share/mantle-bootstrap/final-native-gcc10-cycle.txt" <<'EOF'
claim: bounded final GCC 10 cycle matrix passed
positive: static and dynamic C++17 runtime; libgcc_s; libstdc++ shared runtime; relocated static runtime
negative: malformed C++ rejected without output; absolute shared-runtime dependencies rejected
non_claims: general compiler correctness; provider admission; seed admission; whole-bootstrap correctness
EOF
      echo "final GCC 10.5.0 cycle static/dynamic C++17, libgcc_s, relocation, and rejection matrix passed"
    "%
  ],
  inputs = [
    gcc10_v1,
    musl125,
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "flex-2.6.4-gcc.ncl",
    crunch.derivationFile "bison-3.4.1-gcc.ncl",
    crunch.derivationFile "gmp-6.2.1.ncl",
    crunch.derivationFile "mpfr-4.1.0.ncl",
    crunch.derivationFile "mpc-1.2.1.ncl",
    gcc10_src,
  ],
} | crunch.Derivation
