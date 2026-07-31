# StageX-backed GNU Make compatibility root for later native bootstrap stages.
#
# The protected transition already built and checked this Make output. This
# adapter preserves the historical derivation name without importing provider
# bytes or rebuilding through the old TinyCC chain.

let crunch = import "lib.ncl" in
let stagex_transition = import "stagex-transition-proof-input.ncl" in

{
  name = "make-3.82-tcc",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TRANSITION=
      for candidate in $NIX_STORE/*-mantle-stagex-transition; do
        if [ -d "$candidate" ]; then
          TRANSITION=$candidate
        fi
      done
      test -n "$TRANSITION"
      SOURCE="$TRANSITION/make-stage/runtime/output"
      test -x "$SOURCE/bin/make"
      $BB mkdir -p "$out"
      $BB cp -R "$SOURCE/." "$out/"
      test -x "$out/bin/make"
    "%,
  ],
  inputs = [stagex_transition],
} | crunch.Derivation
