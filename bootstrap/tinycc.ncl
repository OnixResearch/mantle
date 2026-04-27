# Build tinycc 0.9.27 self-hosted from tinycc 0.9.26.
#
# tcc 0.9.27 adds features needed by early GCC (better ELF support,
# archive creation, etc.). Multiple bootstrap passes rebuild tcc and
# the mes libc to gain correctness confidence.
#
# Output: self-hosted tcc 0.9.27 + rebuilt mes libc.
#
# Adapted from live-bootstrap steps/tcc-0.9.27/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc26 = import "tinycc-mes.ncl" in

let tcc27_src = crunch.fetchTarball {
  url = "https://download.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-3ija9gaKo/IKzXsGmFEzEaACxzBN1Im6+MhIv4KI7cw=",
  name = "tcc-0.9.27-src",
} in

{
  name = "tinycc-0.9.27",
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

      STAGE0=$(find_input stage0-posix)
      MES_OUT=$(find_input mes)
      TCC26=$(find_input tinycc-0.9.26)
      TCC27_SRC=$(find_input tcc-0.9.27-src)

      MES_ARCH=x86_64

      export PATH="$TCC26/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      WORK=/tmp/tcc27-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      # fetchTarball strips single top-level dir
      $BB cp -r "$TCC27_SRC" tcc-0.9.27
      $BB chmod -R u+w tcc-0.9.27
      cd tcc-0.9.27

      # Create empty config.h
      : > config.h

      ################################################################
      # Compile tcc-0.9.27 using tcc-0.9.26                         #
      ################################################################
      tcc-0.9.26 \
        -v -static \
        -o tcc-pass1 \
        -D TCC_TARGET_X86_64=1 \
        -D CONFIG_TCCDIR=\"/tmp/tcc-lib\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MES_OUT/lib\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D CONFIG_TCC_LIBPATHS=\"$MES_OUT/lib:$MES_OUT/lib/${MES_ARCH}-mes\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MES_OUT/include/mes-include"'"' \
        -D TCC_LIBGCC=\"$MES_OUT/lib/${MES_ARCH}-mes/libc.a\" \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.27\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc-pass1

      # Self-host: tcc-0.9.27 rebuilds itself
      ./tcc-pass1 \
        -v -static \
        -o tcc \
        -D TCC_TARGET_X86_64=1 \
        -D CONFIG_TCCDIR=\"/tmp/tcc-lib\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MES_OUT/lib\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D CONFIG_TCC_LIBPATHS=\"$MES_OUT/lib:$MES_OUT/lib/${MES_ARCH}-mes\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MES_OUT/include/mes-include"'"' \
        -D TCC_LIBGCC=\"$MES_OUT/lib/${MES_ARCH}-mes/libc.a\" \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.27\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc

      ################################################################
      # Install                                                       #
      ################################################################
      $BB mkdir -p "$out/bin" "$out/lib" "$out/include"

      $BB cp tcc "$out/bin/tcc"
      $BB cp tcc "$out/bin/tcc-0.9.27"

      # Copy mes libc and headers for downstream stages
      $BB cp -r "$MES_OUT/lib/${MES_ARCH}-mes" "$out/lib/${MES_ARCH}-mes"
      $BB cp -r "$MES_OUT/include/mes-include" "$out/include/mes"

      test -x "$out/bin/tcc"
      test -x "$out/bin/tcc-0.9.27"
    "%,
  ],
  inputs = [stage0, mes, tcc26, tcc27_src],
} | crunch.Derivation
