# Spike proof: build an i386-targeting TinyCC 0.9.26 and execute its i386 output.
#
# This follows the earlier Mes/TinyCC handoff than the 0.9.27 cross-smoke: use
# Crunch's Mes-built TinyCC 0.9.26 to build a x86_64-hosted/I386-targeting
# TinyCC 0.9.26, then use that compiler to emit a no-libc native i386 ELF.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc26 = import "tinycc-mes.ncl" in

let tcc26_src = crunch.fetchTarball {
  url = "https://lilypond.org/janneke/tcc/tcc-0.9.26-1147-gee75a10c.tar.gz",
  hash = "sha256-Bkmpev9x8vKh2P9X8Z4VSpcy4BJtzc0Tra3WPImnuhc=",
  name = "tcc-0.9.26-src",
} in

{
  name = "spike-i386-tinycc26-cross-smoke",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
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

      STAGE0=$(find_input stage0-posix)
      TCC26=$(find_input tinycc-0.9.26)
      TCC_SRC=$(find_input tcc-0.9.26-src)
      export PATH="$TCC26/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/spike-i386-tinycc26-cross-smoke
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"
      cd "$WORK"
      $BB cp -r "$TCC_SRC" tcc-0.9.26
      $BB chmod -R u+w tcc-0.9.26
      cd tcc-0.9.26
      : > config.h

      echo 'i386-tcc26: build x86_64-hosted/i386-targeting TinyCC 0.9.26' >&2
      tcc-0.9.26 \
        -v \
        -static \
        -o tcc26-i386 \
        -D BOOTSTRAP=1 \
        -D HAVE_FLOAT=1 \
        -D HAVE_BITFIELD=1 \
        -D HAVE_LONG_LONG=1 \
        -D HAVE_SETJMP=1 \
        -D TCC_TARGET_I386=1 \
        -D CONFIG_TCCDIR=\"/no-runtime/tcc\" \
        -D CONFIG_TCC_CRTPREFIX=\"/no-runtime\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D CONFIG_TCC_LIBPATHS=\"/no-runtime:/no-runtime/tcc\" \
        -D CONFIG_TCC_SYSINCLUDEPATHS=\"/no-runtime/include\" \
        -D TCC_LIBGCC=\"/no-runtime/libc.a\" \
        -D TCC_LIBTCC1=\"libtcc1.a\" \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.26-i386-spike\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc26-i386
      ./tcc26-i386 -version

      $BB cat > exit42.s <<'EOF'
.globl _start
_start:
  movl $1, %eax
  movl $42, %ebx
  int $0x80
EOF

      echo 'i386-tcc26: assemble/link no-libc i386 exit42 executable' >&2
      ./tcc26-i386 -nostdlib -static -Wl,-e,_start -o i386-exit42 exit42.s
      $BB chmod 755 i386-exit42

      set +e
      ./i386-exit42
      rc=$?
      set -e
      echo "i386-tcc26: i386-exit42 rc=$rc"
      test "$rc" = 42

      $BB mkdir -p "$out/bin" "$out/share/spike-i386-tinycc26-cross-smoke"
      $BB cp tcc26-i386 "$out/bin/tcc26-i386"
      $BB cp i386-exit42 "$out/bin/i386-exit42"
      $BB printf 'TinyCC 0.9.26 i386 target produced native i386 executable with rc=42\n' > "$out/share/spike-i386-tinycc26-cross-smoke/result.txt"
      test -x "$out/bin/tcc26-i386"
      test -x "$out/bin/i386-exit42"
    "%,
  ],
  inputs = [stage0, tcc26, tcc26_src],
} | crunch.Derivation
