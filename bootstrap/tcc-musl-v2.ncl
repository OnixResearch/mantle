# Final tcc 0.9.27 rebuilt after self-consistent musl (v2).
#
# This is the definitive tcc-musl compiler for the rest of the
# pre-GCC chain. Built with tcc-musl against the second-pass musl.
#
# Adapted from live-bootstrap steps/tcc-0.9.27 (musl-v2 pass).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc_musl = import "tcc-musl.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in

let tcc27_src = crunch.fetchTarball {
  url = "https://download.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-3ija9gaKo/IKzXsGmFEzEaACxzBN1Im6+MhIv4KI7cw=",
  name = "tcc-0.9.27-src",
} in

{
  name = "tcc-0.9.27-musl-v2",
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

      TCC=$(find_input tcc-0.9.27-musl)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-tcc)
      TCC27_SRC=$(find_input tcc-0.9.27-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/tcc-musl-v2
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$TCC27_SRC" tcc-0.9.27
      $BB chmod -R u+w tcc-0.9.27
      cd tcc-0.9.27

      : > config.h

      tcc \
        -v -static \
        -o tcc-musl-v2 \
        -D TCC_TARGET_X86_64=1 \
        -D CONFIG_TCCDIR=\"$out/lib/tcc\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MUSL/lib\" \
        -D CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\" \
        -D CONFIG_TCC_LIBPATHS=\"$MUSL/lib\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MUSL/include"'"' \
        -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.27\" \
        -D ONE_SOURCE=1 \
        -I "$MUSL/include" \
        tcc.c
      $BB chmod 755 tcc-musl-v2

      # Self-host: rebuild with itself
      ./tcc-musl-v2 \
        -v -static \
        -o tcc \
        -D TCC_TARGET_X86_64=1 \
        -D CONFIG_TCCDIR=\"$out/lib/tcc\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MUSL/lib\" \
        -D CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\" \
        -D CONFIG_TCC_LIBPATHS=\"$MUSL/lib\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MUSL/include"'"' \
        -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.27\" \
        -D ONE_SOURCE=1 \
        -I "$MUSL/include" \
        tcc.c
      $BB chmod 755 tcc

      $BB mkdir -p "$out/bin" "$out/lib/tcc"
      $BB cp tcc "$out/bin/tcc"
      $BB cp tcc "$out/bin/tcc-0.9.27-musl-v2"
      $BB chmod 755 "$out/bin/tcc" "$out/bin/tcc-0.9.27-musl-v2"

      if [ -f libtcc1.a ]; then
        $BB cp libtcc1.a "$out/lib/tcc/"
      fi

      test -x "$out/bin/tcc"
    "%,
  ],
  inputs = [stage0, tcc_musl, musl, make, sed, tcc27_src],
} | crunch.Derivation
