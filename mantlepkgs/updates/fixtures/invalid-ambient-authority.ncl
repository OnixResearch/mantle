let Contracts = import "../contracts.ncl" in
let policy = import "policy.ncl" in
(policy & { allow_ambient_credentials | force = true }) | Contracts.Policy
