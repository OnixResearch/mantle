let Contracts = import "../contracts.ncl" in
let head = import "compatible-head.ncl" in
(head & { store_prefix | force = "/other/store" }) | Contracts.Snapshot
