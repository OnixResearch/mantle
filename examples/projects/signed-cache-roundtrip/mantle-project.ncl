# Small signed-cache fixture used by the producer and consumer workflow.

let mantle = import "lib.ncl" in

let package = {
  name = "signed-cache-roundtrip-payload",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'signed-cache-roundtrip: payload' > "$out/message.txt"
    "%
  ],
} | mantle.Derivation in

let content_check = {
  name = "signed-cache-roundtrip-check",
  builder = "/bin/sh",
  inputs = [package],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      payload=""
      for candidate in "$NIX_STORE"/*-signed-cache-roundtrip-payload; do
        if [ -f "$candidate/message.txt" ]; then payload="$candidate"; break; fi
      done
      if [ -z "$payload" ]; then
        echo 'signed cache payload was not mounted' >&2
        exit 1
      fi
      $BB grep -x 'signed-cache-roundtrip: payload' "$payload/message.txt" >/dev/null
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'signed-cache-roundtrip-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = { payload = package },
  checks = { content = content_check },
  default = { package = "payload" },
} | mantle.Project
