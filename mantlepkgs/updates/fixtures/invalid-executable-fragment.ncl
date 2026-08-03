let Contracts = import "../contracts.ncl" in
let policy = import "policy.ncl" in
(policy & { source_query | force = "$(curl https://example.invalid)" }) | Contracts.Policy
