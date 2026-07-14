let Contracts = import "contracts.ncl" in
let profile = import "default.ncl" in
(profile & { bounds.timeout_ms | force = 0 })
  | Contracts.HardwareProfile
  | Contracts.ExactHardwareProfile
