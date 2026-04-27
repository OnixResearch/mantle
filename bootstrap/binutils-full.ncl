# Build binutils 2.41 using GCC 10 + musl 1.2.5.
#
# Full binutils for the final seed toolchain.
#
# Adapted from live-bootstrap steps/binutils-2.41.

let crunch = import "lib.ncl" in

let gcc10 = import "gcc-10.ncl" in
let musl = import "musl-full.ncl" in
let binutils_old = import "binutils-tcc.ncl" in
let make = import "make-tcc.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let flex = import "flex-2.6.4-musl.ncl" in
let bison = import "bison-3.4.1-musl.ncl" in
let perl = import "perl-5.6.2-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let autoconf = import "autoconf-2.69.ncl" in
let automake = import "automake-1.15.1.ncl" in
let libtool = import "libtool-2.2.4.ncl" in
let tar = import "tar-tcc.ncl" in
let stage0 = import "stage0-posix.ncl" in

let binutils_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.41.tar.xz",
  hash = "sha256-8dGjwSZTk28XKXwkODNezqFVLXDA6Ycf/dLF+RoYGWo=",
  name = "binutils-2.41-src",
} in

{
  name = "binutils-2.41-full",
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

      GCC=$(find_input gcc-10.5.0)
      MUSL=$(find_input musl-1.2.5-full)
      BINUTILS_OLD=$(find_input binutils-2.30-tcc)
      MAKE=$(find_input make-3.82-tcc)
      BASH=$(find_input bash-2.05b-tcc)
      CU=$(find_input coreutils-6.10-musl)
      SED=$(find_input sed-4.0.9-musl)
      M4=$(find_input m4-1.4.7-musl)
      FLEX=$(find_input flex-2.6.4-musl)
      BISON=$(find_input bison-3.4.1-musl)
      PERL=$(find_input perl-5.6.2-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      GAWK=$(find_input gawk-3.0.4-musl)
      AC=$(find_input autoconf-2.69)
      AM=$(find_input automake-1.15.1)
      LT=$(find_input libtool-2.2.4)
      TAR=$(find_input tar-1.12-tcc)
      SRC=$(find_input binutils-2.41-src)

      export PATH="$GCC/bin:$BINUTILS_OLD/bin:$MAKE/bin:$SED/bin:$M4/bin:$FLEX/bin:$BISON/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$AC/bin:$AM/bin:$LT/bin:$TAR/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/binutils-full-build
      $BB mkdir -p "$WORK/build"
      cd "$WORK"

      $BB cp -r "$SRC" binutils-2.41
      $BB chmod -R u+w binutils-2.41

      cd "$WORK/build"

      "$BASH/bin/bash" "$WORK/binutils-2.41/configure" \
        --prefix="$out" \
        --build=x86_64-unknown-linux-musl \
        --host=x86_64-unknown-linux-musl \
        --target=x86_64-unknown-linux-musl \
        --disable-nls \
        --disable-shared \
        --disable-werror \
        --disable-gdb \
        --disable-libdecnumber \
        --disable-readline \
        --disable-sim \
        --disable-gprofng \
        --with-sysroot="$MUSL" \
        CC="$GCC/bin/gcc" \
        CXX="$GCC/bin/g++" \
        CFLAGS="-I$MUSL/include -D_GNU_SOURCE -O2" \
        CXXFLAGS="-I$MUSL/include -D_GNU_SOURCE -O2" \
        LDFLAGS="-static -L$MUSL/lib" \
        AR="$BINUTILS_OLD/bin/ar" \
        RANLIB="$BINUTILS_OLD/bin/ranlib" \
        YACC="bison -y" \
        LEX="flex" \
        MAKEINFO=true \
        2>&1

      make -j1 MAKEINFO=true 2>&1
      make -j1 install MAKEINFO=true 2>&1

      # Verify required tools
      for tool in as ld ar ranlib nm objcopy objdump readelf strip; do
        test -x "$out/bin/$tool" || test -x "$out/bin/x86_64-unknown-linux-musl-$tool" || {
          echo "WARNING: $tool not found" >&2
        }
      done

      test -x "$out/bin/as" || test -x "$out/bin/x86_64-unknown-linux-musl-as" || {
        echo "ERROR: no assembler" >&2; exit 1
      }

      echo "binutils-2.41 build complete"
    "%,
  ],
  inputs = [
    gcc10, musl, binutils_old, make, bash, coreutils, sed, m4, flex,
    bison, perl, grep, diffutils, gawk, autoconf, automake, libtool,
    tar, stage0, binutils_src,
  ],
} | crunch.Derivation
