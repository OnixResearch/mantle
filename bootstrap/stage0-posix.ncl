# StageX-backed POSIX compatibility root for later native bootstrap stages.
#
# The protected Rust transition constructs these utilities before this NCL can
# run. This adapter copies only declared StageX outputs. It does not rebuild or
# authorize the historical pre-StageX seed chain.

let crunch = import "lib.ncl" in
let stagex_transition = import "stagex-transition-proof-input.ncl" in

{
  name = "stage0-posix",
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
      COREUTILS="$TRANSITION/coreutils-stage/runtime/output/bin"
      BASH="$TRANSITION/bash-full-stage/runtime/output/bin/bash-full"
      test -d "$COREUTILS"
      test -x "$BASH"
      $BB mkdir -p "$out/bin"
      for tool in "$COREUTILS"/*; do
        test -x "$tool"
        $BB cp "$tool" "$out/bin/${tool##*/}"
      done
      $BB cp "$BASH" "$out/bin/sh"
      $BB cp "$BASH" "$out/bin/bash"
      $BB chmod 755 "$out/bin"/*
      test -x "$out/bin/sh"
      test -x "$out/bin/cp"
    "%,
  ],
  inputs = [stagex_transition],
} | crunch.Derivation
