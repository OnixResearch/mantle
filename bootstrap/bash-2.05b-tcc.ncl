# StageX-backed Bash compatibility root for generated-source bootstrap stages.
#
# The protected transition builds a bounded non-interactive Bash 2.05b after
# the earlier shell artifact. This adapter preserves the historical derivation
# name and prevents Autoconf/Automake regeneration from reopening Mes inputs.

let crunch = import "lib.ncl" in
let stagex_transition = import "stagex-transition-proof-input.ncl" in

{
  name = "bash-2.05b-tcc",
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
      SOURCE="$TRANSITION/bash-full-stage/runtime/output/bin/bash-full"
      test -x "$SOURCE"
      $BB mkdir -p "$out/bin"
      $BB cp "$SOURCE" "$out/bin/bash"
      $BB cp "$SOURCE" "$out/bin/sh"
      $BB chmod 755 "$out/bin/bash" "$out/bin/sh"
      test -x "$out/bin/bash"
    "%,
  ],
  inputs = [stagex_transition],
} | crunch.Derivation
