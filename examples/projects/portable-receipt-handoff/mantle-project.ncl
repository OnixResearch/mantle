# Payload used by the portable receipt and semantic graph handoff runbook.
let mantle = import "../../../lib/lib.ncl" in

let receipt_payload = {
  name = "portable-receipt-payload",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      "$BB" mkdir -p "$out"
      "$BB" printf '%s\n' 'portable receipt gallery payload' > "$out/payload.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = { payload = receipt_payload },
  default = { package = "payload" },
} | mantle.Project
