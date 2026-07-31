# StageX-backed native musl compatibility root for later bootstrap stages.
#
# The protected StageX provider carries the checked native-musl headers, CRT
# objects, and static libraries. This adapter preserves the historical layout
# without rebuilding through the legacy compiler chain.

let crunch = import "lib.ncl" in
let stagex_provider = import "stagex-provider-proof-input.ncl" in

{
  name = "musl-1.1.24-native-candidate",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-linux-musl
      STAGEX=
      for candidate in $NIX_STORE/*-mantle-stagex-intermediate-provider; do
        if [ -d "$candidate" ]; then
          STAGEX=$candidate
        fi
      done
      test -n "$STAGEX"
      SOURCE="$STAGEX/$TARGET"
      test -d "$SOURCE/include"
      test -s "$SOURCE/lib/libc.a"
      $BB mkdir -p "$out"
      $BB cp -R "$SOURCE/include" "$out/"
      $BB cp -R "$SOURCE/lib" "$out/"
      test -s "$out/lib/crt1.o"
      test -s "$out/lib/libc.a"
    "%,
  ],
  inputs = [stagex_provider],
} | crunch.Derivation
