# StageX-backed binutils compatibility root for later native bootstrap stages.
#
# The protected StageX provider carries the authenticated assembler, linker,
# archive, and inspection tools. This adapter preserves the historical layout
# without rebuilding through the legacy provider chain.

let crunch = import "lib.ncl" in
let stagex_provider = import "stagex-provider-proof-input.ncl" in

{
  name = "binutils-2.30-tcc-source-v1",
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
      $BB mkdir -p "$out/bin" "$out/include" "$out/lib/ldscripts"
      for tool in ar as ld nm objcopy objdump ranlib readelf size strings strip; do
        test -x "$STAGEX/bin/$TARGET-$tool"
        $BB cp "$STAGEX/bin/$TARGET-$tool" "$out/bin/$tool"
        $BB cp "$STAGEX/bin/$TARGET-$tool" "$out/bin/$TARGET-$tool"
      done
      if [ -d "$STAGEX/$TARGET/include-binutils" ]; then
        $BB cp -R "$STAGEX/$TARGET/include-binutils/." "$out/include/"
      fi
      if [ -d "$STAGEX/$TARGET/lib/ldscripts" ]; then
        $BB cp -R "$STAGEX/$TARGET/lib/ldscripts/." "$out/lib/ldscripts/"
      fi
      $BB chmod 755 "$out/bin"/*
      test -x "$out/bin/ld"
      test -x "$out/bin/ar"
    "%,
  ],
  inputs = [stagex_provider],
} | crunch.Derivation
