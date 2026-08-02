let Contracts = import "../contracts.ncl" in
let policy = import "../default.ncl" in

(policy & { allow_local_build = true }) | Contracts.CacheClosurePolicy
