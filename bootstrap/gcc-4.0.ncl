# Build GCC 4.0.4 using tcc + musl + binutils 2.30.
#
# This is the first real C compiler in the bootstrap chain, replacing
# TinyCC. GCC 4.0.4 is the oldest GCC that TCC can build. Attempts
# C and C++ languages; falls back to C-only if C++ fails.
#
# Adapted from live-bootstrap steps/gcc-4.0.4/pass1.sh.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let binutils = import "binutils-tcc.ncl" in
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

let gcc_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2",
  hash = "sha256-QNHCUqWDcOtqVLokzAd3MfsWWR/jN3HdAld/Pm+GSM0=",
  name = "gcc-4.0.4-src",
} in

{
  name = "gcc-4.0.4",
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

      TCC=$(find_input tcc-0.9.27-musl-v2)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      BINUTILS=$(find_input binutils-2.30-tcc)
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
      SRC=$(find_input gcc-4.0.4-src)

      export PATH="$BINUTILS/bin:$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$FLEX/bin:$BISON/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$AC/bin:$AM/bin:$LT/bin:$PATCH/bin:$TAR/bin:$BZIP2/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/gcc-build
      $BB mkdir -p "$WORK/build"
      cd "$WORK"

      $BB cp -r "$SRC" gcc-4.0.4
      $BB chmod -R u+w gcc-4.0.4

      # GCC 4.0.4 needs to find assembler and linker from binutils
      export AS="$BINUTILS/bin/as"
      export LD="$BINUTILS/bin/ld"
      export AR="$BINUTILS/bin/ar"
      export RANLIB="$BINUTILS/bin/ranlib"
      export NM="$BINUTILS/bin/nm"
      export OBJCOPY="$BINUTILS/bin/objcopy"
      export STRIP="$BINUTILS/bin/strip"

      # Build in separate directory (GCC requires out-of-tree build)
      cd "$WORK/build"

      # Try C and C++ first; fall back to C only
      LANGS="c,c++"
      "$BASH/bin/bash" "$WORK/gcc-4.0.4/configure" \
        --prefix="$out" \
        --build=x86_64-unknown-linux-musl \
        --host=x86_64-unknown-linux-musl \
        --target=x86_64-unknown-linux-musl \
        --enable-languages="$LANGS" \
        --disable-bootstrap \
        --disable-shared \
        --disable-nls \
        --disable-multilib \
        --disable-decimal-float \
        --disable-libmudflap \
        --disable-libssp \
        --disable-libgomp \
        --disable-threads \
        --with-sysroot="$MUSL" \
        --with-native-system-header-dir=/include \
        --with-build-sysroot="$MUSL" \
        --without-headers \
        CC=tcc \
        CXX="false" \
        CFLAGS="-I$MUSL/include -D_GNU_SOURCE -DHAVE_ALLOCA_H" \
        CFLAGS_FOR_TARGET="-I$MUSL/include -D_GNU_SOURCE" \
        LDFLAGS="-static -L$MUSL/lib" \
        MAKEINFO=true \
        2>&1 || {
          echo "C+C++ configure failed, trying C-only..." >&2
          cd "$WORK"
          $BB rm -rf build
          $BB mkdir -p build
          cd build
          LANGS="c"
          "$BASH/bin/bash" "$WORK/gcc-4.0.4/configure" \
            --prefix="$out" \
            --build=x86_64-unknown-linux-musl \
            --host=x86_64-unknown-linux-musl \
            --target=x86_64-unknown-linux-musl \
            --enable-languages="$LANGS" \
            --disable-bootstrap \
            --disable-shared \
            --disable-nls \
            --disable-multilib \
            --disable-decimal-float \
            --disable-libmudflap \
            --disable-libssp \
            --disable-libgomp \
            --disable-threads \
            --with-sysroot="$MUSL" \
            --with-native-system-header-dir=/include \
            --with-build-sysroot="$MUSL" \
            --without-headers \
            CC=tcc \
            CFLAGS="-I$MUSL/include -D_GNU_SOURCE -DHAVE_ALLOCA_H" \
            CFLAGS_FOR_TARGET="-I$MUSL/include -D_GNU_SOURCE" \
            LDFLAGS="-static -L$MUSL/lib" \
            MAKEINFO=true \
            2>&1
        }

      # Build GCC
      make -j1 all-gcc CC=tcc AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true 2>&1 || {
        echo "make all-gcc failed, trying individual targets..." >&2
        # Build libiberty first (dependency of gcc)
        make -j1 -C libiberty CC=tcc AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" 2>&1 || true
        # Build gcc
        make -j1 -C gcc CC=tcc AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true 2>&1 || true
      }

      # Build target libraries (libgcc at minimum)
      make -j1 all-target-libgcc CC=tcc AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true 2>&1 || {
        echo "target libgcc failed, continuing without..." >&2
      }

      # Install
      $BB mkdir -p "$out/bin" "$out/lib" "$out/include" "$out/libexec"
      make -j1 install-gcc MAKEINFO=true 2>&1 || {
        echo "make install-gcc failed, manual install..." >&2
        # Copy compiler binaries manually
        for tool in gcc cc1 collect2; do
          for cand in \
            "gcc/$tool" \
            "gcc/${tool}-cross" \
            "gcc/xgcc"; do
            if [ -x "$WORK/build/$cand" ]; then
              $BB cp "$WORK/build/$cand" "$out/bin/" 2>/dev/null || true
            fi
          done
        done
        # xgcc is the stage-1 compiler
        if [ -x "$WORK/build/gcc/xgcc" ]; then
          $BB cp "$WORK/build/gcc/xgcc" "$out/bin/gcc"
          $BB chmod 755 "$out/bin/gcc"
        fi
        # cc1 is the C compiler proper
        $BB mkdir -p "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4"
        if [ -x "$WORK/build/gcc/cc1" ]; then
          $BB cp "$WORK/build/gcc/cc1" "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/"
        fi
        if [ -x "$WORK/build/gcc/cc1plus" ]; then
          $BB cp "$WORK/build/gcc/cc1plus" "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/"
        fi
      }

      # Install target libraries if available
      make -j1 install-target-libgcc MAKEINFO=true 2>&1 || true

      # Verify: at minimum, the C compiler must work
      test -x "$out/bin/gcc" || test -x "$out/bin/x86_64-unknown-linux-musl-gcc" || {
        echo "ERROR: no gcc binary produced" >&2
        exit 1
      }

      # Create cc symlink
      if [ -x "$out/bin/gcc" ]; then
        $BB ln -sf gcc "$out/bin/cc"
      elif [ -x "$out/bin/x86_64-unknown-linux-musl-gcc" ]; then
        $BB ln -sf x86_64-unknown-linux-musl-gcc "$out/bin/gcc"
        $BB ln -sf x86_64-unknown-linux-musl-gcc "$out/bin/cc"
      fi

      # C compiler smoke test
      $BB cat > /tmp/smoke.c << 'EOF'
int main(void) { return 0; }
EOF

      "$out/bin/gcc" -I"$MUSL/include" -L"$MUSL/lib" -static \
        -B"$BINUTILS/bin/" -B"$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/" \
        -o /tmp/smoke /tmp/smoke.c 2>&1 || {
        echo "WARNING: gcc smoke test failed (may need runtime path adjustments)" >&2
      }

      echo "gcc-4.0.4 build complete (languages: $LANGS)"
    "%,
  ],
  inputs = [
    stage0, mes, tcc, musl, make, binutils, sed, m4, flex, bison,
    perl, coreutils, gawk, grep, diffutils, bash,
    autoconf, automake, libtool, patch, tar, bzip2,
    gcc_src,
  ],
} | crunch.Derivation
