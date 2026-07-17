# Negative identity fixture: one declared payload byte sequence changes.
let mantle = import "../../../../lib/lib.ncl" in

{
  name = "hermetic-plan-rebuild-payload",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      "$BB" mkdir -p "$out"
      "$BB" printf '%s\n' 'hermetic-plan-rebuild: changed payload' > "$out/payload.txt"
    "%
  ],
} | mantle.Derivation
