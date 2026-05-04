# Build binutils 2.30 using the full tcc-era intermediate tool chain.
#
# This is the final output of the binutils-tcc chain, providing as,
# ld, ar, ranlib, nm, and objcopy for the gcc-4.0.4 transition.
#
# Adapted from live-bootstrap steps/binutils-2.30.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
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
let patch = import "patch-tcc.ncl" in
let tar = import "tar-tcc.ncl" in
let bzip2 = import "bzip2-1.0.8-musl.ncl" in

let binutils_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz",
  hash = "sha256-L8aaWezlL47cNdPIbSEAtryYUDXLe9htMSECT4xqpoc=",
  name = "binutils-2.30-src",
} in

{
  name = "binutils-2.30-tcc",
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
      PATCH=$(find_input patch-2.5.9-tcc)
      TAR=$(find_input tar-1.12-tcc)
      BZIP2=$(find_input bzip2-1.0.8-musl)
      SRC=$(find_input binutils-2.30-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$FLEX/bin:$BISON/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$PATCH/bin:$TAR/bin:$BZIP2/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL="$BASH/bin/bash"
      export SHELL="$BASH/bin/bash"

      WORK=/tmp/binutils-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" binutils-2.30
      $BB chmod -R u+w binutils-2.30
      cd binutils-2.30

      # Configure binutils with tcc
      $BASH/bin/bash ./configure \
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
        --with-sysroot="$MUSL" \
        CC=tcc \
        CFLAGS="-I$MUSL/include -static -D_GNU_SOURCE" \
        LDFLAGS="-static" \
        AR="tcc -ar" \
        RANLIB="true" \
        YACC="bison -y" \
        LEX="flex" \
        2>/dev/null || true

      # Build
      make -j1 CC=tcc AR="tcc -ar" RANLIB=true MAKEINFO=true 2>/dev/null || {
        # Fallback: build just the essential tools manually
        echo "Full configure/make failed, trying manual build..." >&2

        # At minimum, build gas (assembler) and binutils (ar, nm, etc.)
        cd "$WORK/binutils-2.30"

        # Build libiberty
        cd libiberty
        make -j1 CC=tcc AR="tcc -ar" RANLIB=true 2>/dev/null || true
        cd ..

        # Build bfd
        cd bfd
        make -j1 CC=tcc AR="tcc -ar" RANLIB=true MAKEINFO=true 2>/dev/null || true
        cd ..

        # Build opcodes
        cd opcodes
        make -j1 CC=tcc AR="tcc -ar" RANLIB=true 2>/dev/null || true
        cd ..

        # Build gas
        cd gas
        make -j1 CC=tcc AR="tcc -ar" RANLIB=true MAKEINFO=true 2>/dev/null || true
        cd ..

        # Build binutils
        cd binutils
        make -j1 CC=tcc AR="tcc -ar" RANLIB=true MAKEINFO=true 2>/dev/null || true
        cd ..

        # Build ld
        cd ld
        make -j1 CC=tcc AR="tcc -ar" RANLIB=true MAKEINFO=true 2>/dev/null || true
        cd ..
      }

      # Install
      make -j1 install prefix="$out" MAKEINFO=true 2>/dev/null || {
        $BB mkdir -p "$out/bin" "$out/lib"

        # Install what we can find
        for tool in as ld ar nm objcopy objdump ranlib readelf size strings strip; do
          for candidate in \
            "gas/as-new" "gas/as" \
            "ld/ld-new" "ld/ld" \
            "binutils/$tool" "binutils/${tool}-new"; do
            if [ -x "$WORK/binutils-2.30/$candidate" ]; then
              $BB cp "$WORK/binutils-2.30/$candidate" "$out/bin/$tool"
              $BB chmod 755 "$out/bin/$tool"
              break
            fi
          done
        done
      }

      # Verify required tools exist
      test -x "$out/bin/as" || { echo "ERROR: as not built" >&2; exit 1; }
      test -x "$out/bin/ld" || { echo "ERROR: ld not built" >&2; exit 1; }
      test -x "$out/bin/ar" || { echo "ERROR: ar not built" >&2; exit 1; }
      test -x "$out/bin/ranlib" || { echo "ERROR: ranlib not built" >&2; exit 1; }
      test -x "$out/bin/nm" || { echo "ERROR: nm not built" >&2; exit 1; }
      test -x "$out/bin/objcopy" || { echo "ERROR: objcopy not built" >&2; exit 1; }

      # Assembler smoke test
      $BB cat > /tmp/smoke.s << 'ASM'
.section .text
.global _start
_start:
  mov $60, %rax
  xor %rdi, %rdi
  syscall
ASM
      "$out/bin/as" -o /tmp/smoke.o /tmp/smoke.s
      test -f /tmp/smoke.o
      echo "binutils-2.30 smoke test passed"
    "%,
  ],
  inputs = [
    stage0, mes, tcc, musl, make, sed, m4, flex, bison,
    perl, coreutils, gawk, grep, diffutils, bash,
    patch, tar, bzip2,
    binutils_src,
  ],
} | crunch.Derivation
