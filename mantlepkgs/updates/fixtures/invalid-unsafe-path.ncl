let Contracts = import "../contracts.ncl" in
let policy = import "policy.ncl" in
(policy & {
  mutation | force = (policy.mutation & { relative_path | force = "../escape.json" }),
}) | Contracts.Policy
