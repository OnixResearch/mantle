let Contracts = import "../contracts.ncl" in
let policy = import "policy.ncl" in
(policy & {
  limits | force = (policy.limits & { elapsed_millis | force = 0 }),
}) | Contracts.Policy
