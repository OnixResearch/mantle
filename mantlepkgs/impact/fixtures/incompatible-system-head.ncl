let Contracts = import "../contracts.ncl" in
let head = import "compatible-head.ncl" in
(head & { system | force = "aarch64-linux" }) | Contracts.Snapshot
