let Contracts = import "../contracts.ncl" in
let base = import "compatible-base.ncl" in
(base & {
  catalog_identity_blake3 | force = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
}) | Contracts.Snapshot
