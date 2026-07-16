# Packages the exact Rust delta-substitution example source as a fixed input.

let mantle = import "lib.ncl" in
let demo_source = mantle.fetchurl {
  url = "file://./demo.rs",
  hash = "blake3-1lyMWVTGmP/TM+0VOlo/5RW9EcrZGY+JJ/qFbGCxPus=",
  name = "delta-substitution-demo-source",
} in

let source_package = {
  name = "delta-substitution-example-source",
  builder = "/bin/sh",
  inputs = [demo_source],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      source=""
      for candidate in "$NIX_STORE"/*-delta-substitution-demo-source; do
        if [ -f "$candidate" ]; then source="$candidate"; break; fi
      done
      if [ -z "$source" ]; then
        echo 'delta example source was not mounted' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB cp "$source" "$out/demo.rs"
      $BB grep -q 'DeltaAcceptanceMode::Delta' "$out/demo.rs"
      $BB grep -q 'FullArtifactFallback' "$out/demo.rs"
      $BB grep -q 'MissingSenderChunk' "$out/demo.rs"
    "%
  ],
} | mantle.Derivation in

{
  packages = { source = source_package },
  default = { package = "source" },
} | mantle.Project
