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
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
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
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      BASH=$(find_input bash-2.05b-tcc)
      TAR=$(find_input tar-1.12-tcc)
      BZIP2=$(find_input bzip2-1.0.8-musl)
      SRC=$(find_input binutils-2.30-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$TAR/bin:$BZIP2/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export CC=tcc

      export CFLAGS="-I$MUSL/include -static -D_GNU_SOURCE"
      export LDFLAGS="-static"
      export AR="tcc -ar"
      export RANLIB=true
      export MAKEINFO=true
      export CC_FOR_BUILD=tcc
      export CFLAGS_FOR_BUILD="-I$MUSL/include -static -D_GNU_SOURCE"

      WORK=/tmp/binutils-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      TOOLBIN="$WORK/tools"
      $BB mkdir -p "$TOOLBIN"
      for tool in expr rm ls pwd mkdir rmdir chmod cp mv cat sed grep awk sort uniq tr basename dirname printf test touch ln true; do
        $BB cat > "$TOOLBIN/$tool" <<EOF
#!/bin/sh
exec /bin/busybox $tool "\$@"
EOF
        $BB chmod 755 "$TOOLBIN/$tool"
      done
      $BB cat > "$TOOLBIN/cpp-tcc" <<EOF
#!/bin/sh
last=""
for arg in "\$@"; do last="\$arg"; done
if [ -f "\$last" ] && /bin/busybox grep -q 'Syntax error' "\$last"; then
  /bin/busybox cat "\$last"
  exit 0
fi
exec tcc -E -I$MUSL/include "\$@"
EOF
      $BB chmod 755 "$TOOLBIN/cpp-tcc"
      export PATH="$TOOLBIN:$PATH"
      export CPP="$TOOLBIN/cpp-tcc"

      $BB cp -r "$SRC" binutils-2.30
      $BB chmod -R u+w binutils-2.30
      cd binutils-2.30

      for mf in bfd/Makefile.in binutils/Makefile.in gas/Makefile.in; do
        if [ -f "$mf" ]; then
          $BB sed -i 's/^SUBDIRS = doc po/SUBDIRS =/' "$mf"
        fi
      done

      $BB cat > libiberty/concat.c <<'EOF'
#ifdef HAVE_CONFIG_H
#include "config.h"
#endif
#include <stdlib.h>
#include <string.h>

unsigned long concat_length(const char *first, ...) {
  return first ? (unsigned long)strlen(first) : 0;
}

char *concat_copy(char *dst, const char *first, ...) {
  if (first) strcpy(dst, first); else *dst = 0;
  return dst;
}

char *libiberty_concat_ptr;

char *concat_copy2(const char *first, ...) {
  if (first) strcpy(libiberty_concat_ptr, first); else *libiberty_concat_ptr = 0;
  return libiberty_concat_ptr;
}

char *concat(const char *first, ...) {
  unsigned long n = first ? (unsigned long)strlen(first) : 0;
  char *out = (char *)malloc(n + 1);
  if (!out) abort();
  if (first) strcpy(out, first); else *out = 0;
  return out;
}

char *reconcat(char *optr, const char *first, ...) {
  char *out = concat(first, 0);
  if (optr) free(optr);
  return out;
}
EOF

      config_common() {
        LD=true \
        AR="tcc -ar" \
        CC=tcc \
        CPP="$TOOLBIN/cpp-tcc" \
        CFLAGS="-I$MUSL/include -static -D_GNU_SOURCE -DBUILDFIXED=1 -DDYNAMIC_CRC_TABLE=1" \
        LDFLAGS="-static" \
        RANLIB=true \
        MAKEINFO=true \
        /bin/sh ./configure \
          --disable-nls \
          --enable-deterministic-archives \
          --enable-64-bit-bfd \
          --disable-shared \
          --disable-werror \
          --build=x86_64-unknown-linux-gnu \
          --host=x86_64-unknown-linux-musl \
          --target=x86_64-unknown-linux-gnu \
          --program-prefix="" \
          --prefix="$out" \
          --libdir="$out/lib" \
          --with-sysroot= \
          --srcdir=. \
          --enable-compressed-debug-sections=all \
          lt_cv_sys_max_cmd_len=32768
      }

      # Configure the subdirectories in live-bootstrap order.  This avoids the
      # top-level recursive make adding unsupported options to the early GNU make
      # and lets us patch generated Makefiles at the exact failing seams.
      for dir in intl libiberty opcodes bfd binutils gas ld; do
        echo "configure: $dir" >&2
        cd "$WORK/binutils-2.30/$dir"
        config_common || { echo "ERROR: configure failed in $dir" >&2; exit 1; }
        cd "$WORK/binutils-2.30"
      done

      if [ -f libiberty/Makefile ]; then
        $BB sed -i \
          -e 's/regex\.c//g' \
          -e 's#\./regex\.\$[(]objext[)] ##g' \
          -e 's#\./regex\.\$[(]objext[)]##g' \
          -e 's/-W -Wall -Wwrite-strings -Wc++-compat -Wstrict-prototypes -Wshadow=local//g' \
          libiberty/Makefile
        for name in regex cplus-dem cp-demangle md5 sha1; do
          $BB cat > "libiberty/$name-placeholder.c" <<EOF
void placeholder(void) {}
EOF
          tcc -c "libiberty/$name-placeholder.c" -o "libiberty/$name.o"
        done
      fi

      test -f bfd/bfd-in2.h || { echo "ERROR: missing generated bfd/bfd-in2.h" >&2; exit 1; }
      test -f bfd/libbfd.h || { echo "ERROR: missing generated bfd/libbfd.h" >&2; exit 1; }

      for dir in libiberty bfd opcodes binutils gas ld; do
        echo "build: $dir" >&2
        make -j1 -C "$dir" \
          tooldir="$out" \
          CC=tcc \
          CPP="$TOOLBIN/cpp-tcc" \
          CFLAGS="-I$MUSL/include -static -D_GNU_SOURCE -DBUILDFIXED=1 -DDYNAMIC_CRC_TABLE=1" \
          CPPFLAGS="-DPLUGIN_LITTLE_ENDIAN" \
          LDFLAGS="-static" \
          AR="tcc -ar" \
          RANLIB=true \
          MAKEINFO=true || {
            echo "ERROR: build failed in $dir" >&2
            for log in libiberty/config.log bfd/config.log opcodes/config.log gas/config.log ld/config.log; do
              if [ -f "$WORK/binutils-2.30/$log" ]; then
                echo "===== $log tail =====" >&2
                $BB tail -220 "$WORK/binutils-2.30/$log" >&2 || true
              fi
            done
            exit 1
          }
      done

      # Install
      $BB mkdir -p "$out/bin" "$out/lib"
      for dir in libiberty bfd opcodes binutils gas ld; do
        echo "install: $dir" >&2
        make -j1 -C "$dir" \
          tooldir="$out" \
          DESTDIR= \
          prefix="$out" \
          MAKEINFO=true \
          install || true
      done

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
  inputs = [stage0, mes, tcc, musl, make, sed, m4, grep, diffutils, bash, tar, bzip2, binutils_src],
} | crunch.Derivation
