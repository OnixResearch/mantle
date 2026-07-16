# Select only the development output from a multi-output dependency.
#
# Usage:
#   mantle build examples/selected-output.ncl --no-substitute

let mantle = import "lib.ncl" in

let example_sdk = {
  name = "example-sdk",
  builder = "/bin/sh",
  outputs = ["out", "dev"],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      $BB mkdir -p "$out/bin" "$dev/include"
      $BB printf '%s\n' 'runtime payload' > "$out/bin/example-tool"
      $BB printf '%s\n' '#define EXAMPLE_API "selected-dev-output"' > "$dev/include/example_api.h"
    "%
  ],
} | mantle.Derivation in

{
  name = "selected-output-consumer",
  builder = "/bin/sh",
  inputs = [mantle.select example_sdk "dev"],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      header=""

      for candidate in "$NIX_STORE"/*; do
        if [ -f "$candidate/include/example_api.h" ]; then
          header="$candidate/include/example_api.h"
          break
        fi
      done

      if [ -z "$header" ]; then
        echo 'selected development output was not mounted' >&2
        exit 1
      fi

      $BB mkdir -p "$out"
      $BB cat "$header" > "$out/selected-header.txt"
    "%
  ],
} | mantle.Derivation
