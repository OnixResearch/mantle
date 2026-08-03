let Contracts = import "../contracts.ncl" in
let head = import "compatible-base.ncl" in
(head & { catalog_identity_blake3 | force = "not-a-blake3-digest" }) | Contracts.Snapshot
