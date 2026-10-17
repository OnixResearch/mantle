# Source-built CMake for the full-source Rust provider route.
#
# This host tool is compiled by the admitted normalized GCC/musl/binutils
# provider and uses only declared source-built make, Linux UAPI headers, and
# BusyBox utilities. Its receipt is still separate from Rust-provider admission.

let crunch = import "lib.ncl" in
let native_provider = crunch.derivationFile "seed-full-toolchain.ncl" in
let gnumake = crunch.derivationFile "make-4.4.1-gcc10.ncl" in
let linux_headers = crunch.derivationFile "linux-headers-6.6-gcc10.ncl" in
let busybox = crunch.derivationFile "busybox-1.37.0-gcc10.ncl" in
let pin = import "pins/generated/cmake.json" in
let cmake_src = crunch.fetchTarball {
  url = pin.artifacts.source.url,
  hash = pin.artifacts.source.hash,
  name = "cmake-" ++ pin.version ++ "-src",
} in

{
  name = "cmake-" ++ pin.version ++ "-full-source-gcc10-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      JOB_COUNT=1
      TOOL_PROBE_TIMEOUT_SECONDS=30
      WORK=/tmp/cmake-%{pin.version}-full-source
      TARGET=x86_64-linux-musl

      find_input() {
        input_suffix=$1
        matched_input=
        match_count=0
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            matched_input=$input_path
            match_count=$((match_count + 1))
          fi
        done
        if [ "$match_count" -ne 1 ]; then
          echo "ERROR: expected exactly one declared input $input_suffix, got $match_count" >&2
          exit 1
        fi
        echo "$matched_input"
      }

      PROVIDER=$(find_input full-source-seed-toolchain)
      MAKE=$(find_input make-4.4.1-full-source-gcc10-v1)
      LINUX_HEADERS=$(find_input linux-headers-6.6-full-source-gcc10-v3)
      BUSYBOX=$(find_input busybox-1.37.0-full-source-gcc10-v1)
      SRC=$(find_input cmake-%{pin.version}-src)
      SOURCE_ROOT="$SRC"
      if [ -f "$SRC/cmake-%{pin.version}/bootstrap" ]; then
        SOURCE_ROOT="$SRC/cmake-%{pin.version}"
      fi

      for required_executable in \
        "$PROVIDER/bin/$TARGET-gcc" \
        "$PROVIDER/bin/$TARGET-g++" \
        "$PROVIDER/bin/ar" \
        "$MAKE/bin/make" \
        "$BUSYBOX/bin/busybox"; do
        if [ ! -x "$required_executable" ]; then
          echo "ERROR: required CMake construction executable is missing: $required_executable" >&2
          exit 1
        fi
      done
      if [ ! -f "$LINUX_HEADERS/include/linux/fs.h" ]; then
        echo "ERROR: declared Linux headers lack include/linux/fs.h: $LINUX_HEADERS" >&2
        exit 1
      fi
      if [ ! -f "$SOURCE_ROOT/bootstrap" ]; then
        echo "ERROR: authenticated CMake source has no bootstrap script at either bounded root: $SRC" >&2
        exit 1
      fi

      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK/tools"
      for applet in awk basename cat chmod cmp cp cut date dirname echo env expr find grep head hostname id install ln ls mkdir mv od paste printf pwd readlink realpath rm sed sh sleep sort tail tee test touch tr uname uniq wc xargs; do
        $BB ln -sf "$BUSYBOX/bin/busybox" "$WORK/tools/$applet"
      done
      $BB ln -sf "$MAKE/bin/make" "$WORK/tools/make"
      $BB ln -sf "$MAKE/bin/make" "$WORK/tools/gmake"
      $BB ln -sf "$PROVIDER/bin/$TARGET-gcc" "$WORK/tools/cc"
      $BB ln -sf "$PROVIDER/bin/$TARGET-g++" "$WORK/tools/c++"
      $BB ln -sf "$PROVIDER/bin/ar" "$WORK/tools/ar"
      $BB ln -sf "$PROVIDER/bin/ranlib" "$WORK/tools/ranlib"
      $BB ln -sf "$PROVIDER/bin/ld" "$WORK/tools/ld"

      export PATH="$WORK/tools"
      export CONFIG_SHELL="$BUSYBOX/bin/sh"
      export SHELL="$BUSYBOX/bin/sh"
      export MAKE="$MAKE/bin/make"
      export CC="$PROVIDER/bin/$TARGET-gcc"
      export CXX="$PROVIDER/bin/$TARGET-g++"
      export AR="$PROVIDER/bin/ar"
      export RANLIB="$PROVIDER/bin/ranlib"
      export LD="$PROVIDER/bin/ld"
      export CFLAGS="-O2 -I$LINUX_HEADERS/include"
      export CXXFLAGS="-O2 -I$LINUX_HEADERS/include"
      export LDFLAGS="-static"
      export SOURCE_DATE_EPOCH=1

      $BB cp -R "$SOURCE_ROOT" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"
      "$BUSYBOX/bin/sh" ./bootstrap \
        --prefix="$out" \
        --parallel="$JOB_COUNT" \
        -- \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_EXE_LINKER_FLAGS=-static \
        -DCMAKE_SHARED_LINKER_FLAGS= \
        -DCMAKE_MODULE_LINKER_FLAGS= \
        -DCMAKE_USE_OPENSSL=OFF \
        -DBUILD_CursesDialog=OFF
      "$MAKE" -j"$JOB_COUNT"
      "$MAKE" install

      CMZLIB_BUILD="$WORK/src/Utilities/cmzlib"
      test -f "$CMZLIB_BUILD/libcmzlib.a"
      test -f "$CMZLIB_BUILD/zlib.h"
      test -f "$CMZLIB_BUILD/zconf.h"
      test -f "$CMZLIB_BUILD/cm_zlib_mangle.h"
      $BB mkdir -p "$out/include" "$out/lib"
      $BB cp "$CMZLIB_BUILD/libcmzlib.a" "$out/lib/libz.a"
      $BB cp "$CMZLIB_BUILD/zlib.h" "$CMZLIB_BUILD/zconf.h" "$CMZLIB_BUILD/cm_zlib_mangle.h" "$out/include/"

      test -x "$out/bin/cmake"
      test -x "$out/bin/ctest"
      test -x "$out/bin/cpack"
      "$out/bin/cmake" --version | "$BUSYBOX/bin/grep" -q 'cmake version %{pin.version}'

      $BB mkdir -p "$WORK/positive"
      $BB cat > "$WORK/positive/CMakeLists.txt" <<'EOF'
cmake_minimum_required(VERSION 3.20)
project(mantle_cmake_smoke C)
add_executable(mantle-cmake-smoke main.c)
EOF
      $BB cat > "$WORK/positive/main.c" <<'EOF'
#include <stdio.h>
int main(void) { return puts("mantle-source-cmake-ok") < 0; }
EOF
      "$out/bin/cmake" -S "$WORK/positive" -B "$WORK/positive-build" \
        -DCMAKE_C_COMPILER="$PROVIDER/bin/$TARGET-gcc" \
        -DCMAKE_MAKE_PROGRAM="$MAKE" \
        -DCMAKE_EXE_LINKER_FLAGS=-static
      "$out/bin/cmake" --build "$WORK/positive-build" --parallel "$JOB_COUNT"
      "$WORK/positive-build/mantle-cmake-smoke" | "$BUSYBOX/bin/grep" -q mantle-source-cmake-ok

      $BB mkdir -p "$WORK/rejected"
      $BB printf '%s\n' 'this is not valid CMake syntax (' > "$WORK/rejected/CMakeLists.txt"
      if "$BUSYBOX/bin/timeout" "$TOOL_PROBE_TIMEOUT_SECONDS" "$out/bin/cmake" -S "$WORK/rejected" -B "$WORK/rejected-build"; then
        echo "ERROR: source-built CMake accepted malformed project input" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-build/Makefile"

      RELOCATED_CMAKE="$WORK/relocated-cmake"
      $BB cp -R "$out" "$RELOCATED_CMAKE"
      "$RELOCATED_CMAKE/bin/cmake" --version | "$BUSYBOX/bin/grep" -q 'cmake version %{pin.version}'

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/full-source-rust-host-cmake.txt" <<'EOF'
claim: bounded source-built CMake host-tool matrix passed
positive: version, C configure/build/run, bundled static zlib archive, copied-tree version
negative: malformed CMake project rejected without generated Makefile
source_authority: authenticated CMake %{pin.version} source
native_authority: admitted full-source GCC/musl/binutils provider
non_claims: CMake correctness, Rust provider completion, whole-bootstrap correctness
EOF
      echo "source-built CMake %{pin.version} host-tool matrix passed"
    "%
  ],
  inputs = [native_provider, gnumake, linux_headers, busybox, cmake_src],
} | crunch.Derivation
