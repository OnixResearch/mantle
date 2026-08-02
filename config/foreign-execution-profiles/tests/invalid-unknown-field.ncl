let Contracts = import "../contracts.ncl" in
let profile = import "../guix.ncl" in
(profile & { ambient_shell = "/bin/sh" })
  | Contracts.ExecutionProfile
  | Contracts.ExactExecutionProfile
