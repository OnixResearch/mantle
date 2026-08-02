let Contracts = import "../contracts.ncl" in
let profile = import "../guix.ncl" in
(profile & { allow_setid = true })
  | Contracts.ExecutionProfile
  | Contracts.ExactExecutionProfile
