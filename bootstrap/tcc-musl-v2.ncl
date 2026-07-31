# StageX-backed TinyCC musl-v2 compatibility root.
#
# The protected StageX provider is newer than this historical handoff and
# carries the required compiler/runtime shape. This adapter keeps the old
# derivation name while preventing a return to the pre-StageX provider chain.

let crunch = import "lib.ncl" in
let stagex_provider = import "stagex-provider-proof-input.ncl" in

{
  name = "tcc-0.9.27-musl-v2",
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
      $BB mkdir -p "$out/bin" "$out/lib"
      $BB cp "$STAGEX/bin/$TARGET-tcc" "$out/bin/tcc"
      $BB cp "$STAGEX/bin/$TARGET-tcc" "$out/bin/tcc-0.9.27-musl-v2"
      $BB cp -R "$STAGEX/lib/tcc" "$out/lib/"
      $BB chmod 755 "$out/bin"/*
      test -s "$out/lib/tcc/libtcc1.a"
    "%,
  ],
  inputs = [stagex_provider],
} | crunch.Derivation
