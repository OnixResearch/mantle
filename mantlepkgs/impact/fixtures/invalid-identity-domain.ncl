let Contracts = import "../contracts.ncl" in
let head = import "compatible-head.ncl" in
(head & { identity_domain | force = "unreviewed.identity.domain" }) | Contracts.Snapshot
