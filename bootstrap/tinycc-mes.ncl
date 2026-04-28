# Build tinycc 0.9.26 using the mes C compiler (mescc).
#
# This is the first real C compiler in the chain. mescc (running on
# mes-m2) compiles tcc 0.9.26 from C source. The resulting tcc is then
# used to rebuild itself multiple times (boot0 → boot1 → boot2 → final)
# to gain confidence in correctness.
#
# Output: tcc-0.9.26 binary + mes libc rebuilt with tcc.
#
# Adapted from live-bootstrap steps/tcc-0.9.26/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in

let tcc_src = crunch.fetchTarball {
  url = "https://lilypond.org/janneke/tcc/tcc-0.9.26-1147-gee75a10c.tar.gz",
  hash = "sha256-Bkmpev9x8vKh2P9X8Z4VSpcy4BJtzc0Tra3WPImnuhc=",
  name = "tcc-0.9.26-src",
} in

{
  name = "tinycc-0.9.26",
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
      TCC_SRC=$(find_input tcc-0.9.26-src)

      # mes environment
      export MES_STACK=15000000
      export MES_ARENA=30000000
      export MES_MAX_ARENA=30000000
      MES="$MES_OUT/bin/mes-m2"
      MESCC="$MES_OUT/bin/mescc.scm"

      MES_ARCH=x86_64
      MES_LIB="$MES_OUT/lib/${MES_ARCH}-mes"
      MES_TCC_BASE_ADDRESS=0x08048000

      export PATH="$STAGE0/bin:$MES_OUT/bin:$PATH"
      export MES_PREFIX="$MES_OUT"
      export GUILE_LOAD_PATH="$MES_OUT/mes/module"

      WORK=/tmp/tcc-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      # fetchTarball strips single top-level dir
      $BB cp -r "$TCC_SRC" tcc-0.9.26
      $BB chmod -R u+w tcc-0.9.26

      # Create empty config.h
      : > tcc-0.9.26/config.h

      cd tcc-0.9.26

      ################################################################
      # Step 1: Compile tcc with mescc (Scheme-based C compiler)     #
      ################################################################
      "$MES" --no-auto-compile -e main "$MESCC" -- \
        -S -o tcc.s \
        -I "$MES_OUT/include/mes-include" \
        -D BOOTSTRAP=1 \
        -D HAVE_LONG_LONG=1 \
        -I . \
        -D TCC_TARGET_X86_64=1 \
        -D inline= \
        -D CONFIG_TCCDIR=\"/tmp/tcc-lib\" \
        -D CONFIG_SYSROOT=\"/\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MES_LIB\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MES_OUT/include/mes-include"'"' \
        -D TCC_LIBGCC=\"$MES_LIB/libc.a\" \
        -D CONFIG_TCC_LIBTCC1_MES=0 \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.26\" \
        -D ONE_SOURCE=1 \
        tcc.c

      # Link through mescc so mes-provided crt/libc+tcc objects are applied.
      "$MES" --no-auto-compile -e main "$MESCC" -- \
        --base-address "$MES_TCC_BASE_ADDRESS" \
        -o tcc-mes \
        -L "$MES_LIB" \
        tcc.s \
        -l c+tcc
      $BB chmod 755 tcc-mes
      ./tcc-mes -version

      ################################################################
      # Step 2: Bootstrap passes -- each tcc rebuilds itself         #
      ################################################################

      # tcc-mes → tcc-boot0 (first self-compile)
      ./tcc-mes \
        -s -o tcc-boot0 \
        -I "$MES_OUT/include/mes-include" \
        -D BOOTSTRAP=1 -D HAVE_LONG_LONG=1 \
        -I . \
        -D TCC_TARGET_X86_64=1 \
        -D inline= \
        -D CONFIG_TCCDIR=\"/tmp/tcc-lib\" \
        -D CONFIG_SYSROOT=\"/\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MES_LIB\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MES_OUT/include/mes-include"'"' \
        -D TCC_LIBGCC=\"$MES_LIB/libc.a\" \
        -D CONFIG_TCC_LIBTCC1_MES=0 \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.26\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc-boot0

      # tcc-boot0 → tcc-boot1
      ./tcc-boot0 \
        -s -o tcc-boot1 \
        -I "$MES_OUT/include/mes-include" \
        -D BOOTSTRAP=1 -D HAVE_LONG_LONG=1 \
        -I . \
        -D TCC_TARGET_X86_64=1 \
        -D inline= \
        -D CONFIG_TCCDIR=\"/tmp/tcc-lib\" \
        -D CONFIG_SYSROOT=\"/\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MES_LIB\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MES_OUT/include/mes-include"'"' \
        -D TCC_LIBGCC=\"$MES_LIB/libc.a\" \
        -D CONFIG_TCC_LIBTCC1_MES=0 \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.26\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc-boot1

      # tcc-boot1 → final tcc
      ./tcc-boot1 \
        -s -o tcc \
        -I "$MES_OUT/include/mes-include" \
        -D BOOTSTRAP=1 -D HAVE_LONG_LONG=1 \
        -I . \
        -D TCC_TARGET_X86_64=1 \
        -D inline= \
        -D CONFIG_TCCDIR=\"/tmp/tcc-lib\" \
        -D CONFIG_SYSROOT=\"/\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MES_LIB\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MES_OUT/include/mes-include"'"' \
        -D TCC_LIBGCC=\"$MES_LIB/libc.a\" \
        -D CONFIG_TCC_LIBTCC1_MES=0 \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.26\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc

      ################################################################
      # Install                                                       #
      ################################################################
      $BB mkdir -p "$out/bin" "$out/lib"

      $BB cp tcc "$out/bin/tcc-0.9.26"
      $BB cp tcc "$out/bin/tcc"

      # Copy mes libc for tcc to use
      $BB cp -r "$MES_LIB" "$out/lib/${MES_ARCH}-mes"
      $BB cp -r "$MES_OUT/include" "$out/include"

      test -x "$out/bin/tcc"
      test -x "$out/bin/tcc-0.9.26"
    "%,
  ],
  inputs = [stage0, mes, tcc_src],
} | crunch.Derivation
