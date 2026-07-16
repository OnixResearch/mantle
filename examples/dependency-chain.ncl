# Build one derivation, mount it as an explicit input, and consume its output.
#
# Usage:
#   mantle build examples/dependency-chain.ncl --no-substitute

let mantle = import "lib.ncl" in

let message_data = {
  name = "example-message-data",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'Hello from the producer.' > "$out/message.txt"
    "%
  ],
} | mantle.Derivation in

{
  name = "dependency-chain",
  builder = "/bin/sh",
  inputs = [message_data],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      message_input=""

      for candidate in "$NIX_STORE"/*-example-message-data; do
        if [ -f "$candidate/message.txt" ]; then
          message_input="$candidate"
          break
        fi
      done

      if [ -z "$message_input" ]; then
        echo 'declared message input was not mounted' >&2
        exit 1
      fi

      $BB mkdir -p "$out"
      $BB cat "$message_input/message.txt" > "$out/result.txt"
      $BB printf '%s\n' 'Consumed by the dependency-chain root.' >> "$out/result.txt"
    "%
  ],
} | mantle.Derivation
