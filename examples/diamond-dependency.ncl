# Build a diamond graph: two branches reuse one shared input before joining.
#
# Usage:
#   mantle build examples/diamond-dependency.ncl --no-substitute

let mantle = import "lib.ncl" in

let shared = {
  name = "diamond-shared",
  builder = "/bin/sh",
  args = ["-c", "printf '%s\\n' 'shared foundation' > \"$out\""],
} | mantle.Derivation in

let branch = fun branch_name branch_message => {
  name = branch_name,
  builder = "/bin/sh",
  inputs = [shared],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      shared_input=""

      for candidate in "$NIX_STORE"/*-diamond-shared; do
        if [ -f "$candidate" ]; then
          shared_input="$candidate"
          break
        fi
      done

      if [ -z "$shared_input" ]; then
        echo 'shared diamond input was not mounted' >&2
        exit 1
      fi

      $BB cat "$shared_input" > "$out"
      $BB printf '%s\n' '%{branch_message}' >> "$out"
    "%
  ],
} | mantle.Derivation in

let left = branch "diamond-left" "left branch" in
let right = branch "diamond-right" "right branch" in

{
  name = "diamond-result",
  builder = "/bin/sh",
  inputs = [left, right],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      left_input=""
      right_input=""

      for candidate in "$NIX_STORE"/*-diamond-left; do
        if [ -f "$candidate" ]; then
          left_input="$candidate"
          break
        fi
      done
      for candidate in "$NIX_STORE"/*-diamond-right; do
        if [ -f "$candidate" ]; then
          right_input="$candidate"
          break
        fi
      done

      if [ -z "$left_input" ] || [ -z "$right_input" ]; then
        echo 'both diamond branches must be mounted' >&2
        exit 1
      fi

      $BB mkdir -p "$out"
      $BB cat "$left_input" > "$out/graph.txt"
      $BB cat "$right_input" >> "$out/graph.txt"
    "%
  ],
} | mantle.Derivation
