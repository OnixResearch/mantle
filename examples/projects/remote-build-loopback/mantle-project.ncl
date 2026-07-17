# Seed-free payload for the local stdio remote-build workflow.
let mantle = import "../../../lib/lib.ncl" in

let remote_payload = {
  name = "remote-loopback-payload",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      "$BB" mkdir -p "$out"
      "$BB" printf '%s\n' 'remote loopback gallery payload' > "$out/payload.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = { payload = remote_payload },
  default = { package = "payload" },
} | mantle.Project
