let Contracts = import "../contracts.ncl" in
let policy = import "policy.ncl" in
(policy & { max_packages | force = 0 }) | Contracts.Policy
