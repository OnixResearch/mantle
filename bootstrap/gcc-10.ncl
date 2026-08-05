# GCC 10.5.0 C/C++ final-native parity row over the receipted GCC 4.7 line.
#
# The separately declared source derivation regenerates the contracted source
# set with source-built generators. This claim is bounded to the row matrix.

let crunch = import "lib.ncl" in

let gcc10_src = crunch.derivationFile "gcc-10.5.0-regenerated-source.ncl" in

{
  name = "gcc-10.5.0-musl-gcc47-v2",
  builder = "/bin/sh",
  # GCC embeds its configured prefix in drivers and internal tools.
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      SEED_GCC_VERSION=4.7.4
      OUTPUT_GCC_VERSION=10.5.0
      JOB_COUNT=1
      BUILD_FLAGS='-O2 -fno-common'
      WORK=/tmp/gcc-10.5.0-musl-gcc47
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

      GCC47=$(find_input gcc-4.7.4-musl-gcc40-v1)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v17)
      MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      GREP=$(find_input grep-2.4-gcc-v10)
      GAWK=$(find_input gawk-3.0.4-gcc-v4)
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
      $BB ln -sf "$MAKE/bin/make" "$TOOLBIN/make"
      export PATH="$TOOLBIN:$BINUTILS/bin:$STAGE0/bin"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export MAKEFLAGS=

      $BB cat > "$TOOLBIN/gcc47" <<EOF
#!/bin/sh
export MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as"
export MANTLE_GCC_LINKER="$BINUTILS/bin/ld"
export MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a"
export MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a"
export MANTLE_GCC_LINK_LIBGCC="$GCC47/lib/gcc/$TARGET/$SEED_GCC_VERSION/libgcc.a"
export MANTLE_GCC_LINK_CRT="$MUSL/lib"
exec "$GCC47/bin/gcc" -B"$GCC47/bin/" -B"$GCC47/lib/gcc/$TARGET/$SEED_GCC_VERSION/" "\$@"
EOF
      $BB cat > "$TOOLBIN/gxx47" <<EOF
#!/bin/sh
export MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as"
export MANTLE_GCC_LINKER="$BINUTILS/bin/ld"
export MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a"
export MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a"
export MANTLE_GCC_LINK_LIBGCC="$GCC47/lib/gcc/$TARGET/$SEED_GCC_VERSION/libgcc.a"
export MANTLE_GCC_LINK_CRT="$MUSL/lib"
exec "$GCC47/bin/g++" -B"$GCC47/bin/" -B"$GCC47/lib/gcc/$TARGET/$SEED_GCC_VERSION/" "\$@"
EOF
      $BB chmod 755 "$TOOLBIN/gcc47" "$TOOLBIN/gxx47"
      HOST_CC="$TOOLBIN/gcc47"
      HOST_CXX="$TOOLBIN/gxx47"

      $BB cat > "$out/bin/as" <<EOF
#!/bin/sh
exec "\${MANTLE_GCC_ASSEMBLER:-$BINUTILS/bin/as}" "\$@"
EOF
      $BB chmod 755 "$out/bin/as"

      $BB cat > "$out/bin/ld" <<EOF
#!/bin/sh
libc_archive="\${MANTLE_GCC_LINK_LIBC:-$MUSL/lib/libc.a}"
libm_archive="\${MANTLE_GCC_LINK_LIBM:-$MUSL/lib/libm.a}"
libgcc_archive="\${MANTLE_GCC_LINK_LIBGCC:-$GCC47/lib/gcc/$TARGET/$SEED_GCC_VERSION/libgcc.a}"
zlib_archive="\${MANTLE_GCC_LINK_ZLIB:-$WORK/build/zlib/libz.a}"
crt_dir="\${MANTLE_GCC_LINK_CRT:-$MUSL/lib}"
linker="\${MANTLE_GCC_LINKER:-$BINUTILS/bin/ld}"
link_arguments=
for link_argument in "\$@"; do
  case "\$link_argument" in
    -lgcc|-lgcc_eh) link_arguments="\$link_arguments \$libgcc_archive" ;;
    -lgcc_s) ;;
    crt1.o|Scrt1.o|rcrt1.o|crti.o|crtn.o) link_arguments="\$link_arguments \$crt_dir/\$link_argument" ;;
    -lc) link_arguments="\$link_arguments \$libc_archive" ;;
    -lm) link_arguments="\$link_arguments \$libm_archive" ;;
    -lpthread|-ldl|-lrt|-lutil) link_arguments="\$link_arguments \$libc_archive" ;;
    -lz) link_arguments="\$link_arguments \$zlib_archive" ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
echo "gcc10-final-ld: \$linker \$link_arguments" >&2
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
      $BB mkdir -p "$WORK/build"
      cd "$WORK/build"

      CC="$HOST_CC" \
      CXX="$HOST_CXX" \
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
        --enable-static \
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
        --disable-threads \
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
        MAKEINFO=true

      make -j"$JOB_COUNT" all-gcc \
        CC="$HOST_CC" CXX="$HOST_CXX" \
        AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" \
        CFLAGS="-std=gnu11 $BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        CXXFLAGS="-std=gnu++11 $BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        BOOT_CFLAGS="$BUILD_FLAGS -I$MUSL/include -I$WORK/src/zlib" \
        GMPLIBS="$MPC/lib/libmpc.a $MPFR/lib/libmpfr.a $GMP/lib/libgmp.a" \
        MAKEINFO=true || {
          build_status=$?
          if [ -f "$WORK/build/gcc/config.log" ]; then
            $BB cat "$WORK/build/gcc/config.log" >&2
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
      "$BINUTILS/bin/readelf" -h "$WORK/positive-c" | "$GREP/bin/grep" -q 'ELF64'

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
      MANTLE_GCC_LINK_LIBGCC="$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/libgcc.a" \
        "$out/bin/g++" -std=c++17 -B"$out/bin/" \
          -B"$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/" \
          -I"$MUSL/include" -static "$WORK/positive.cpp" -o "$WORK/positive-cxx"
      test -s "$WORK/positive-cxx"
      $BB chmod 755 "$WORK/positive-cxx"
      "$WORK/positive-cxx"

      $BB printf '%s\n' 'int main( {' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected-c.o"
      if "$out/bin/gcc" -B"$out/bin/" \
        -B"$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/" \
        -I"$MUSL/include" -c "$WORK/rejected.c" -o "$WORK/rejected-c.o"; then
        echo "ERROR: GCC 10 accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-c.o"

      $BB printf '%s\n' 'template <typename T> int main(' > "$WORK/rejected.cpp"
      $BB rm -f "$WORK/rejected-cxx.o"
      if "$out/bin/g++" -std=c++17 -B"$out/bin/" \
        -B"$out/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/" \
        -I"$MUSL/include" -c "$WORK/rejected.cpp" -o "$WORK/rejected-cxx.o"; then
        echo "ERROR: GCC 10 accepted malformed C++" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-cxx.o"

      RELOCATED_GCC="$WORK/relocated-gcc10"
      $BB cp -R "$out" "$RELOCATED_GCC"
      MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
      MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
      MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
      MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
      MANTLE_GCC_LINK_LIBGCC="$RELOCATED_GCC/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/libgcc.a" \
      MANTLE_GCC_LINK_CRT="$MUSL/lib" \
        "$RELOCATED_GCC/bin/g++" -std=c++17 \
          -B"$RELOCATED_GCC/bin/" \
          -B"$RELOCATED_GCC/lib/gcc/$TARGET/$OUTPUT_GCC_VERSION/" \
          -I"$MUSL/include" -static "$WORK/positive.cpp" \
          -o "$WORK/relocated-positive"
      test -s "$WORK/relocated-positive"
      $BB chmod 755 "$WORK/relocated-positive"
      "$WORK/relocated-positive"

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cp "$WORK/src/.mantle-regeneration/gcc-10.5.0-files.txt" "$out/share/mantle-bootstrap/gcc10-regenerated-files.txt"
      $BB cat > "$out/share/mantle-bootstrap/final-native-gcc10-row.txt" <<'EOF'
claim: bounded final-native GCC 10 row behavior matrix passed
positive: C static runtime; C++17 exceptions,RTTI,new,delete,dynamic_cast; relocated C++ runtime
negative: malformed C rejected without output; malformed C++ rejected without output
non_claims: general compiler correctness; provider admission; seed admission; whole-bootstrap correctness
EOF
      echo "GCC 10.5.0 final-native row C/C++17 runtime, ELF, relocation, and rejection matrix passed"
    "%
  ],
  inputs = [
    crunch.derivationFile "gcc-4.7.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "gmp-6.2.1.ncl",
    crunch.derivationFile "mpfr-4.1.0.ncl",
    crunch.derivationFile "mpc-1.2.1.ncl",
    gcc10_src,
  ],
} | crunch.Derivation
