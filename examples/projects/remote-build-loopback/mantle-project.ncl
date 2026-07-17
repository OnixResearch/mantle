# Seed-free payloads for the local production stdio remote-build workflow.
# r[impl examples.resumable_remote_transfer_workflow]
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

let transfer_payload_block_size_bytes = 65536 in
let transfer_payload_block_count = 4 in
let resumable_payload = {
  name = "remote-resumable-payload",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      "$BB" mkdir -p "$out"
      "$BB" printf '%s\n' 'resumable remote transfer gallery payload' > "$out/payload.txt"
      "$BB" dd if=/dev/zero of="$out/payload.bin" bs=%{transfer_payload_block_size_bytes} count=%{transfer_payload_block_count} 2>/dev/null
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    payload = remote_payload,
    resumable-payload = resumable_payload,
  },
  default = { package = "payload" },
} | mantle.Project
