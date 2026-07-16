# Packages the exact Rust release/witness handoff example source as a fixed input.

let mantle = import "lib.ncl" in
let demo_source = mantle.fetchurl {
  url = "file://./demo.rs",
  hash = "blake3-lL6DrPK6mK/xR6zsgZiPPDulaVyZlCARs2kwnjJ05CQ=",
  name = "release-witness-handoff-demo-source",
} in

let source_package = {
  name = "release-witness-handoff-example-source",
  builder = "/bin/sh",
  inputs = [demo_source],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      source=""
      for candidate in "$NIX_STORE"/*-release-witness-handoff-demo-source; do
        if [ -f "$candidate" ]; then source="$candidate"; break; fi
      done
      if [ -z "$source" ]; then
        echo 'release witness example source was not mounted' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB cp "$source" "$out/demo.rs"
      $BB grep -q 'VerificationDirectory' "$out/demo.rs"
      $BB grep -q 'evaluate_policy' "$out/demo.rs"
      $BB grep -q 'ReleaseRevocations' "$out/demo.rs"
    "%
  ],
} | mantle.Derivation in

{
  packages = { source = source_package },
  default = { package = "source" },
} | mantle.Project
