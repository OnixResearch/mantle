# StageX-backed self-hosted TinyCC compatibility root.
#
# The protected StageX provider carries the checked self-hosted compiler,
# runtime archive, headers, and patched source. This adapter preserves the
# historical layout without invoking the pre-StageX compiler chain.

let crunch = import "lib.ncl" in
let stagex_provider = import "stagex-provider-proof-input.ncl" in

{
  name = "tcc-0.9.27-musl-selfhost",
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
      test -x "$STAGEX/bin/$TARGET-tcc"
      test -d "$STAGEX/lib/tcc"
      $BB mkdir -p "$out/bin" "$out/lib" "$out/share"
      $BB cp "$STAGEX/bin/$TARGET-tcc" "$out/bin/tcc"
      $BB cp "$STAGEX/bin/$TARGET-tcc" "$out/bin/tcc-0.9.27-musl-selfhost"
      $BB cp -R "$STAGEX/lib/tcc" "$out/lib/"
      $BB cp -R "$STAGEX/share/mantle-stagex/tcc-source-patched" "$out/share/"
      $BB chmod 755 "$out/bin"/*
      test -s "$out/lib/tcc/libtcc1.a"
    "%,
  ],
  inputs = [stagex_provider],
} | crunch.Derivation
