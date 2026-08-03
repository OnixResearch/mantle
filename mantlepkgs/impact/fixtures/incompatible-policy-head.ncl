let Contracts = import "../contracts.ncl" in
let head = import "compatible-head.ncl" in
(head & {
  conversion_policy_identity_blake3 | force = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
}) | Contracts.Snapshot
