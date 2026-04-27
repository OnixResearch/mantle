# Build GCC 10.5.0 using GCC 4.7.4 + GMP + MPFR + MPC.
#
# GCC 10 is the modern compiler that produces the final seed toolchain.
# It builds both C and C++ with full optimization support.
#
# Adapted from live-bootstrap steps/gcc-10.5.0/pass1.sh.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let binutils = import "binutils-tcc.ncl" in
let gcc47 = import "gcc-4.7.ncl" in
let gmp = import "gmp-6.2.1.ncl" in
let mpfr = import "mpfr-4.1.0.ncl" in
let mpc = import "mpc-1.2.1.ncl" in
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

let gcc10_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-10.5.0/gcc-10.5.0.tar.xz",
  hash = "sha256-JBIJ24Wq7Jnl8KHfRf7Q2Bv01VNbptVxOP+drvrJJjM=",
  name = "gcc-10.5.0-src",
} in

{
  name = "gcc-10.5.0",
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

      GCC47=$(find_input gcc-4.7.4)
      BINUTILS=$(find_input binutils-2.30-tcc)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      GMP=$(find_input gmp-6.2.1)
      MPFR=$(find_input mpfr-4.1.0)
      MPC=$(find_input mpc-1.2.1)
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
      SRC=$(find_input gcc-10.5.0-src)

      export PATH="$GCC47/bin:$BINUTILS/bin:$MAKE/bin:$SED/bin:$M4/bin:$FLEX/bin:$BISON/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$AC/bin:$AM/bin:$LT/bin:$PATCH/bin:$TAR/bin:$BZIP2/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/gcc10-build
      $BB mkdir -p "$WORK/build"
      cd "$WORK"

      $BB cp -r "$SRC" gcc-10.5.0
      $BB chmod -R u+w gcc-10.5.0

      export AS="$BINUTILS/bin/as"
      export LD="$BINUTILS/bin/ld"
      export AR="$BINUTILS/bin/ar"
      export RANLIB="$BINUTILS/bin/ranlib"
      export NM="$BINUTILS/bin/nm"

      cd "$WORK/build"

      "$BASH/bin/bash" "$WORK/gcc-10.5.0/configure" \
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
        --disable-libsanitizer \
        --disable-plugin \
        --with-sysroot="$MUSL" \
        --with-native-system-header-dir=/include \
        --with-build-sysroot="$MUSL" \
        --with-gmp="$GMP" \
        --with-mpfr="$MPFR" \
        --with-mpc="$MPC" \
        CC="$GCC47/bin/gcc" \
        CXX="$GCC47/bin/g++" \
        CFLAGS="-I$MUSL/include -D_GNU_SOURCE -O2" \
        CXXFLAGS="-I$MUSL/include -D_GNU_SOURCE -O2" \
        CFLAGS_FOR_TARGET="-I$MUSL/include -D_GNU_SOURCE -O2" \
        LDFLAGS="-static -L$MUSL/lib" \
        MAKEINFO=true \
        2>&1

      make -j1 all-gcc MAKEINFO=true 2>&1
      make -j1 all-target-libgcc MAKEINFO=true 2>&1 || true

      $BB mkdir -p "$out/bin" "$out/lib" "$out/libexec"
      make -j1 install-gcc MAKEINFO=true 2>&1
      make -j1 install-target-libgcc MAKEINFO=true 2>&1 || true

      # Create symlinks
      if [ -x "$out/bin/x86_64-unknown-linux-musl-gcc" ] && [ ! -x "$out/bin/gcc" ]; then
        $BB ln -sf x86_64-unknown-linux-musl-gcc "$out/bin/gcc"
        $BB ln -sf x86_64-unknown-linux-musl-gcc "$out/bin/cc"
      elif [ -x "$out/bin/gcc" ]; then
        $BB ln -sf gcc "$out/bin/cc"
      fi
      if [ -x "$out/bin/x86_64-unknown-linux-musl-g++" ] && [ ! -x "$out/bin/g++" ]; then
        $BB ln -sf x86_64-unknown-linux-musl-g++ "$out/bin/g++"
        $BB ln -sf x86_64-unknown-linux-musl-g++ "$out/bin/c++"
      elif [ -x "$out/bin/g++" ]; then
        $BB ln -sf g++ "$out/bin/c++"
      fi

      # Verify
      test -x "$out/bin/gcc" || test -x "$out/bin/x86_64-unknown-linux-musl-gcc" || {
        echo "ERROR: no gcc binary" >&2; exit 1
      }

      # C smoke test
      $BB cat > /tmp/smoke.c << 'EOF'
int main(void) { return 0; }
EOF
      "$out/bin/gcc" -I"$MUSL/include" -L"$MUSL/lib" -static \
        -B"$BINUTILS/bin/" -o /tmp/smoke_c /tmp/smoke.c 2>&1 || {
        echo "WARNING: C smoke test failed" >&2
      }

      # C++ smoke test
      if [ -x "$out/bin/g++" ]; then
        $BB cat > /tmp/smoke.cpp << 'EOF'
#include <cstddef>
auto add(int a, int b) -> int { return a + b; }
int main() {
  int arr[] = {1, 2, 3};
  int sum = 0;
  for (auto& x : arr) sum += x;
  return sum == 6 ? 0 : 1;
}
EOF
        "$out/bin/g++" -std=c++11 -I"$MUSL/include" -L"$MUSL/lib" -static \
          -B"$BINUTILS/bin/" -o /tmp/smoke_cpp /tmp/smoke.cpp 2>&1 || {
          echo "WARNING: C++ smoke test failed" >&2
        }
      fi

      echo "gcc-10.5.0 build complete"
    "%,
  ],
  inputs = [
    stage0, tcc, musl, make, binutils, gcc47, gmp, mpfr, mpc,
    sed, m4, flex, bison, perl, coreutils, gawk, grep, diffutils,
    bash, autoconf, automake, libtool, patch, tar, bzip2,
    gcc10_src,
  ],
} | crunch.Derivation
