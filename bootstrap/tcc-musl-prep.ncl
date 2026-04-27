# Patched tcc 0.9.27 prepared for building musl.
#
# This pass recompiles tcc with config suitable for musl libc headers
# instead of mes libc. It is the bridge compiler that builds the first musl.
#
# Adapted from live-bootstrap steps/tcc-0.9.27 (musl-prep pass).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in

let tcc27_src = crunch.fetchTarball {
  url = "https://download.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-3ija9gaKo/IKzXsGmFEzEaACxzBN1Im6+MhIv4KI7cw=",
  name = "tcc-0.9.27-src",
} in

{
  name = "tcc-0.9.27-musl-prep",
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

      TCC=$(find_input tinycc-0.9.27)
      MES_OUT=$(find_input mes)
      STAGE0=$(find_input stage0-posix)
      MAKE=$(find_input make-3.82-tcc)
      SED=$(find_input sed-4.0.9-tcc)
      TCC27_SRC=$(find_input tcc-0.9.27-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      MES_ARCH=x86_64
      WORK=/tmp/tcc-musl-prep
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$TCC27_SRC" tcc-0.9.27
      $BB chmod -R u+w tcc-0.9.27
      cd tcc-0.9.27

      : > config.h

      # Build tcc configured for musl-compatible include paths.
      # At this point we still use mes libc for linking, but point
      # include paths to where musl headers will be installed.
      tcc \
        -v -static \
        -o tcc-musl-prep \
        -D TCC_TARGET_X86_64=1 \
        -D CONFIG_TCCDIR=\"/tmp/tcc-lib\" \
        -D CONFIG_TCC_CRTPREFIX=\"/tmp/musl-out/lib\" \
        -D CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\" \
        -D CONFIG_TCC_LIBPATHS=\"$MES_OUT/lib:$MES_OUT/lib/${MES_ARCH}-mes:/tmp/musl-out/lib\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"/tmp/musl-out/include"'"' \
        -D TCC_LIBGCC=\"$MES_OUT/lib/${MES_ARCH}-mes/libc.a\" \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.27\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc-musl-prep

      $BB mkdir -p "$out/bin" "$out/lib" "$out/include"
      $BB cp tcc-musl-prep "$out/bin/tcc"
      $BB cp tcc-musl-prep "$out/bin/tcc-musl-prep"
      $BB chmod 755 "$out/bin/tcc" "$out/bin/tcc-musl-prep"

      # Carry mes libc for linking until musl is built
      $BB cp -r "$MES_OUT/lib/${MES_ARCH}-mes" "$out/lib/${MES_ARCH}-mes"
      $BB cp -r "$MES_OUT/include/mes-include" "$out/include/mes"

      test -x "$out/bin/tcc"
    "%,
  ],
  inputs = [stage0, mes, tcc, make, sed, tcc27_src],
} | crunch.Derivation
