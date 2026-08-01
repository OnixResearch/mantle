# Diagnostic GCC-built musl successor for the full-source seed provider.
#
# This stage deliberately consumes the isolated GCC pass2 artifact and the
# fresh StageX provider. It remains outside bootstrap/seed.ncl until the
# compiler is rebuilt against this libc and the complete provider admission
# surface passes.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let gcc = crunch.derivationFile "gcc-4.0-native.ncl" in
let stagex_provider = import "stagex-provider-proof-input.ncl" in
let musl_src = crunch.derivationFile "musl-1.1.24-src.ncl" in

{
  name = "musl-1.1.24-gcc-pass3-v5",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=4.0.4
      JOB_COUNT=1
      GCC_COMPAT_FLAGS="-fno-asynchronous-unwind-tables -fno-unwind-tables -fno-common -D__STDC__=1"

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then
            echo "$candidate"
            return
          fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      GCC=$(find_input gcc-4.0.4-native-gas-v45)
      STAGEX=$(find_input mantle-stagex-intermediate-provider)
      PREDECESSOR_MUSL="$STAGEX/x86_64-linux-musl"
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input musl-1.1.24-src)

      WORK=/tmp/musl-gcc-pass3
      TOOLBIN="$WORK/tools"
      BINUTILS="$WORK/binutils"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN" "$BINUTILS/bin"
      for tool in ar as ld nm objcopy objdump ranlib readelf size strings strip; do
        $BB ln -s "$STAGEX/bin/x86_64-linux-musl-$tool" "$BINUTILS/bin/$tool"
      done
      for applet in awk basename cat chmod cp dirname echo expr find grep head ln mkdir mv od printf pwd rm rmdir sed sh sort test touch tr true uname wc; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$TOOLBIN:$BINUTILS/bin:$MAKE/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh

      $BB cp "$PREDECESSOR_MUSL/lib/libc.a" "$WORK/predecessor-libc.a"
      $BB chmod u+rw "$WORK/predecessor-libc.a"
      "$BINUTILS/bin/ar" s "$WORK/predecessor-libc.a"
      "$BINUTILS/bin/ar" t "$WORK/predecessor-libc.a" > "$WORK/predecessor-libc-members.txt"
      test -s "$WORK/predecessor-libc-members.txt"

      $BB cat > "$TOOLBIN/as" <<EOF
#!/bin/sh
assembler_arguments=
for assembler_argument in "\$@"; do
  case "\$assembler_argument" in
    -Qy) ;;
    *) assembler_arguments="\$assembler_arguments \$assembler_argument" ;;
  esac
done
exec "$BINUTILS/bin/as" \$assembler_arguments
EOF
      $BB chmod 755 "$TOOLBIN/as"

      $BB cat > "$TOOLBIN/ld" <<EOF
#!/bin/sh
libc_archive="\${MANTLE_GCC_LINK_LIBC:-$WORK/predecessor-libc.a}"
crt_dir="\${MANTLE_GCC_LINK_CRT:-$PREDECESSOR_MUSL/lib}"
link_arguments=
for link_argument in "\$@"; do
  case "\$link_argument" in
    -lgcc|-lgcc_eh|*/libgcc.a|*/libtcc1.a|*/crtbegin*.o|*/crtend*.o|crtbegin*.o|crtend*.o) ;;
    crt1.o|crti.o|crtn.o) link_arguments="\$link_arguments \$crt_dir/\$link_argument" ;;
    -lc) link_arguments="\$link_arguments \$libc_archive" ;;
    -lm) ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
compat_object="$WORK/pass3-link-compat.o"
if [ "\${MANTLE_GCC_CLEAN_LINK:-0}" = 1 ]; then compat_object=; fi
exec "$BINUTILS/bin/ld" \$link_arguments \$compat_object
EOF
      $BB chmod 755 "$TOOLBIN/ld"

      GCC_WRAPPER="$TOOLBIN/gcc"
      $BB cat > "$GCC_WRAPPER" <<EOF
#!/bin/sh
exec "$GCC/bin/gcc" \
  -B"$GCC/libexec/gcc/$TARGET/$GCC_VERSION/" \
  -B"$TOOLBIN/" \
  $GCC_COMPAT_FLAGS "\$@"
EOF
      $BB chmod 755 "$GCC_WRAPPER"

      $BB cat > "$WORK/pass3-link-compat.c" <<'EOF'
typedef union {
  struct { unsigned int low; int high; } words;
  long long signed_value;
  unsigned long long unsigned_value;
} mantle_double_word;

float __floatundisf(unsigned long long value)
{
  mantle_double_word bits;
  long double converted;
  bits.unsigned_value = value;
  if (bits.words.high >= 0) return (float) bits.signed_value;
  converted = (long double) bits.signed_value;
  converted += 18446744073709551616.0;
  return (float) converted;
}

enum { MANTLE_PAGE_SIZE_BYTES = 4096 };
int getpagesize(void) { return MANTLE_PAGE_SIZE_BYTES; }
int __lsysinfo(void *information) { (void)information; return -1; }
int sem_init(void *semaphore, int shared, unsigned int value) { (void)semaphore; (void)shared; (void)value; return -1; }
int sem_wait(void *semaphore) { (void)semaphore; return -1; }
int sem_post(void *semaphore) { (void)semaphore; return -1; }
int sem_destroy(void *semaphore) { (void)semaphore; return 0; }
EOF
      "$GCC_WRAPPER" -c "$WORK/pass3-link-compat.c" -o "$WORK/pass3-link-compat.o"
      test -s "$WORK/pass3-link-compat.o"

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
      test -f configure || { echo "ERROR: musl configure script missing" >&2; exit 1; }

      CC="$GCC_WRAPPER" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ar s" \
      ./configure \
        --host="$TARGET" \
        --disable-shared \
        --prefix="$out" \
        --libdir="$out/lib" \
        --includedir="$out/include"

      make -j"$JOB_COUNT" \
        CROSS_COMPILE= \
        CC="$GCC_WRAPPER" \
        AR="$BINUTILS/bin/ar" \
        RANLIB="$BINUTILS/bin/ar s" \
        CFLAGS_AUTO= \
        CFLAGS="$GCC_COMPAT_FLAGS"
      make install
      $BB mkdir -p "$out/usr"
      $BB ln -s ../include "$out/usr/include"
      $BB ln -s ../lib "$out/usr/lib"

      test -s "$out/lib/libc.a"
      test -s "$out/lib/crt1.o"
      test -s "$out/lib/crti.o"
      test -s "$out/lib/crtn.o"
      test -f "$out/include/semaphore.h"
      test -f "$out/usr/include/semaphore.h"
      test -s "$out/usr/lib/libc.a"
      "$BINUTILS/bin/ar" t "$out/lib/libc.a" > "$WORK/gcc-libc-members.txt"
      test -s "$WORK/gcc-libc-members.txt"

      $BB cat > "$WORK/gcc-musl-smoke.c" <<'EOF'
#include <semaphore.h>
#include <stdio.h>
#include <unistd.h>

enum {
  FAILURE_PAGE_SIZE = 11,
  FAILURE_SEMAPHORE_INIT = 12,
  FAILURE_SEMAPHORE_WAIT = 13,
  FAILURE_SEMAPHORE_POST = 14,
  FAILURE_SEMAPHORE_VALUE = 15,
  FAILURE_SEMAPHORE_DESTROY = 16,
  FAILURE_STDIO = 17
};

int main(void)
{
  sem_t semaphore;
  int value = 0;
  if (getpagesize() <= 0) return FAILURE_PAGE_SIZE;
  if (sem_init(&semaphore, 0, 1) != 0) return FAILURE_SEMAPHORE_INIT;
  if (sem_wait(&semaphore) != 0) return FAILURE_SEMAPHORE_WAIT;
  if (sem_post(&semaphore) != 0) return FAILURE_SEMAPHORE_POST;
  if (sem_getvalue(&semaphore, &value) != 0 || value != 1) return FAILURE_SEMAPHORE_VALUE;
  if (sem_destroy(&semaphore) != 0) return FAILURE_SEMAPHORE_DESTROY;
  if (fputs("gcc-musl-smoke-ok\n", stdout) < 0) return FAILURE_STDIO;
  return 0;
}
EOF
      MANTLE_GCC_LINK_LIBC="$out/lib/libc.a" \
      MANTLE_GCC_LINK_CRT="$out/lib" \
      MANTLE_GCC_CLEAN_LINK=1 \
      "$GCC_WRAPPER" -I"$out/include" -static \
        "$WORK/gcc-musl-smoke.c" -o "$WORK/gcc-musl-smoke"
      test -s "$WORK/gcc-musl-smoke"
      $BB chmod 755 "$WORK/gcc-musl-smoke"
      smoke_output=$("$WORK/gcc-musl-smoke")
      test "$smoke_output" = gcc-musl-smoke-ok
      "$BINUTILS/bin/readelf" -h "$WORK/gcc-musl-smoke" > "$WORK/gcc-musl-header.txt"
      $BB grep -q 'ELF' "$WORK/gcc-musl-header.txt"

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected.o"
      if "$GCC_WRAPPER" -I"$out/include" -c "$WORK/rejected.c" -o "$WORK/rejected.o"; then
        echo "ERROR: GCC-built musl stage accepted malformed C" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB mkdir -p "$out/share/musl-gcc-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: GCC-built musl requires a compiler rebuild against this libc and complete provider checks' \
        > "$out/share/musl-gcc-frontier/NON_ADMISSION"
      echo "musl-1.1.24 GCC pass3 runtime checks passed; not provider admission"
    "%
  ],
  inputs = [stage0, make, gcc, stagex_provider, musl_src],
} | crunch.Derivation
