# Build GCC 4.7.4 using GCC 4.0.4 + musl + binutils 2.30.
#
# gcc-4.7.4 adds C++11 support. This is the second GCC in the ladder
# and the first that can build modern GCC (10.x).
#
# Adapted from live-bootstrap steps/gcc-4.7.4/pass1.sh.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let binutils = import "binutils-tcc.ncl" in
let gcc4 = import "gcc-4.0.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let flex = import "flex-2.6.4-musl.ncl" in
let bison = import "bison-3.4.1-musl.ncl" in
let perl = import "perl-5.6.2-musl.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let autoconf = import "autoconf-2.69.ncl" in
let automake = import "automake-1.15.1.ncl" in
let libtool = import "libtool-2.2.4.ncl" in
let patch = import "patch-tcc.ncl" in
let tar = import "tar-tcc.ncl" in
let bzip2 = import "bzip2-1.0.8-musl.ncl" in

let gcc47_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/gcc-4.7.4/sources and pass1.sh
  # first-consumer: bootstrap/gcc-4.7.ncl
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2",
  hash = "sha256-7ffdeULrk9WbESRDdRXHT1DoOvrThEBIbozmder5xTc=",
  name = "gcc-4.7.4-src",
} in

{
  name = "gcc-4.7.4",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for d in $NIX_STORE/*-"$1"; do
          if [ -d "$d" ]; then echo "$d"; return; fi
        done
        echo "ERROR: input $1 not found" >&2; exit 1
      }

      GCC4=$(find_input gcc-4.0.4)
      BINUTILS=$(find_input binutils-2.30-tcc)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      M4=$(find_input m4-1.4.7-musl)
      FLEX=$(find_input flex-2.6.4-musl)
      BISON=$(find_input bison-3.4.1-musl)
      PERL=$(find_input perl-5.6.2-musl)
      CU=$(find_input coreutils-6.10-musl)
      GAWK=$(find_input gawk-3.0.4-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      BASH=$(find_input bash-2.05b-tcc)
      AC=$(find_input autoconf-2.69)
      AM=$(find_input automake-1.15.1)
      LT=$(find_input libtool-2.2.4)
      PATCH=$(find_input patch-2.5.9-tcc)
      TAR=$(find_input tar-1.12-tcc)
      BZIP2=$(find_input bzip2-1.0.8-musl)
      SRC=$(find_input gcc-4.7.4-src)

      # gcc-4.0.4 is now the bootstrap compiler
      export PATH="$GCC4/bin:$BINUTILS/bin:$MAKE/bin:$SED/bin:$M4/bin:$FLEX/bin:$BISON/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$AC/bin:$AM/bin:$LT/bin:$PATCH/bin:$TAR/bin:$BZIP2/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/gcc47-build
      $BB mkdir -p "$WORK/build"
      cd "$WORK"

      $BB cp -r "$SRC" gcc-4.7.4
      $BB chmod -R u+w gcc-4.7.4

      export AS="$BINUTILS/bin/as"
      export LD="$BINUTILS/bin/ld"
      export AR="$BINUTILS/bin/ar"
      export RANLIB="$BINUTILS/bin/ranlib"
      export NM="$BINUTILS/bin/nm"
      export OBJCOPY="$BINUTILS/bin/objcopy"
      export STRIP="$BINUTILS/bin/strip"

      cd "$WORK/build"

      # GCC 4.7.4 must preserve the C++ provider contract; do not fall back
      # to a C-only compiler and claim this part complete.
      "$BASH/bin/bash" "$WORK/gcc-4.7.4/configure" \
        --prefix="$out" \
        --build=x86_64-unknown-linux-musl \
        --host=x86_64-unknown-linux-musl \
        --target=x86_64-unknown-linux-musl \
        --enable-languages=c,c++ \
        --disable-bootstrap \
        --disable-shared \
        --disable-nls \
        --disable-multilib \
        --disable-decimal-float \
        --disable-libmudflap \
        --disable-libssp \
        --disable-libgomp \
        --disable-threads \
        --disable-libquadmath \
        --disable-libatomic \
        --disable-libstdcxx-pch \
        --with-sysroot="$MUSL" \
        --with-native-system-header-dir=/include \
        --with-build-sysroot="$MUSL" \
        CC="$GCC4/bin/gcc" \
        CXX="$GCC4/bin/g++" \
        CFLAGS="-I$MUSL/include -D_GNU_SOURCE -O2" \
        CXXFLAGS="-I$MUSL/include -D_GNU_SOURCE -O2" \
        CFLAGS_FOR_TARGET="-I$MUSL/include -D_GNU_SOURCE -O2" \
        LDFLAGS="-static -L$MUSL/lib" \
        MAKEINFO=true \
        2>&1 || { echo "ERROR: gcc-4.7 C/C++ configure failed" >&2; exit 1; }

      # Build GCC and target libgcc; both are part of the output contract.
      make -j1 all-gcc MAKEINFO=true 2>&1 || { echo "ERROR: gcc-4.7 all-gcc failed" >&2; exit 1; }
      make -j1 all-target-libgcc MAKEINFO=true 2>&1 || { echo "ERROR: gcc-4.7 all-target-libgcc failed" >&2; exit 1; }

      # Install
      $BB mkdir -p "$out/bin" "$out/lib" "$out/include" "$out/libexec"
      make -j1 install-gcc MAKEINFO=true 2>&1 || { echo "ERROR: gcc-4.7 install-gcc failed" >&2; exit 1; }
      make -j1 install-target-libgcc MAKEINFO=true 2>&1 || { echo "ERROR: gcc-4.7 install-target-libgcc failed" >&2; exit 1; }

      # Create symlinks
      if [ -x "$out/bin/gcc" ]; then
        $BB ln -sf gcc "$out/bin/cc"
      elif [ -x "$out/bin/x86_64-unknown-linux-musl-gcc" ]; then
        $BB ln -sf x86_64-unknown-linux-musl-gcc "$out/bin/gcc"
        $BB ln -sf x86_64-unknown-linux-musl-gcc "$out/bin/cc"
      fi
      if [ -x "$out/bin/g++" ]; then
        $BB ln -sf g++ "$out/bin/c++"
      elif [ -x "$out/bin/x86_64-unknown-linux-musl-g++" ]; then
        $BB ln -sf x86_64-unknown-linux-musl-g++ "$out/bin/g++"
        $BB ln -sf x86_64-unknown-linux-musl-g++ "$out/bin/c++"
      fi
      test -x "$out/bin/cc" || { echo "ERROR: installed cc missing" >&2; exit 1; }
      test -x "$out/bin/c++" || { echo "ERROR: installed c++ missing" >&2; exit 1; }
      test -x "$out/libexec/gcc/x86_64-unknown-linux-musl/4.7.4/cc1" || { echo "ERROR: installed cc1 missing" >&2; exit 1; }
      test -x "$out/libexec/gcc/x86_64-unknown-linux-musl/4.7.4/cc1plus" || { echo "ERROR: installed cc1plus missing" >&2; exit 1; }
      find "$out" -name libgcc.a -type f | $BB grep -q libgcc.a || { echo "ERROR: installed libgcc.a missing" >&2; exit 1; }

      # Verify: C and C++ compiler entrypoints must exist.
      test -x "$out/bin/gcc" || test -x "$out/bin/x86_64-unknown-linux-musl-gcc" || {
        echo "ERROR: no gcc binary produced" >&2; exit 1
      }
      test -x "$out/bin/g++" || test -x "$out/bin/x86_64-unknown-linux-musl-g++" || {
        echo "ERROR: no g++ binary produced" >&2; exit 1
      }

      # C smoke test
      $BB cat > /tmp/smoke.c << 'EOF'
int main(void) { return 0; }
EOF
      "$out/bin/gcc" -I"$MUSL/include" -L"$MUSL/lib" -static \
        -B"$BINUTILS/bin/" \
        -B"$out/libexec/gcc/x86_64-unknown-linux-musl/4.7.4/" \
        -o /tmp/smoke_c /tmp/smoke.c 2>&1 || {
        echo "ERROR: C smoke test failed" >&2; exit 1
      }
      test -x /tmp/smoke_c || { echo "ERROR: C smoke output missing" >&2; exit 1; }

      # C++ smoke test
      $BB cat > /tmp/smoke.cpp << 'EOF'
int main() { return 0; }
EOF
      "$out/bin/g++" -I"$MUSL/include" -L"$MUSL/lib" -static \
        -B"$BINUTILS/bin/" \
        -B"$out/libexec/gcc/x86_64-unknown-linux-musl/4.7.4/" \
        -o /tmp/smoke_cpp /tmp/smoke.cpp 2>&1 || {
        echo "ERROR: C++ smoke test failed" >&2; exit 1
      }
      test -x /tmp/smoke_cpp || { echo "ERROR: C++ smoke output missing" >&2; exit 1; }

      # C++11 smoke test
      $BB cat > /tmp/smoke11.cpp << 'EOF'
#include <cstddef>
auto add(int a, int b) -> int { return a + b; }
int main() {
  static_assert(sizeof(int) >= 4, "int too small");
  int arr[] = {1, 2, 3};
  int sum = 0;
  for (auto x : arr) sum += x;
  return sum == 6 ? 0 : 1;
}
EOF
      "$out/bin/g++" -std=c++11 -I"$MUSL/include" -L"$MUSL/lib" -static \
        -B"$BINUTILS/bin/" \
        -B"$out/libexec/gcc/x86_64-unknown-linux-musl/4.7.4/" \
        -o /tmp/smoke11 /tmp/smoke11.cpp 2>&1 || {
        echo "ERROR: C++11 smoke test failed" >&2; exit 1
      }
      test -x /tmp/smoke11 || { echo "ERROR: C++11 smoke output missing" >&2; exit 1; }

      echo "gcc-4.7.4 build complete"
    "%,
  ],
  inputs = [
    stage0, mes, tcc, musl, make, binutils, gcc4, sed, m4, flex, bison,
    perl, coreutils, gawk, grep, diffutils, bash,
    autoconf, automake, libtool, patch, tar, bzip2,
    gcc47_src,
  ],
} | crunch.Derivation
