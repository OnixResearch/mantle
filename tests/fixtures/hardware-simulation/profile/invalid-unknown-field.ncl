let Contracts = import "contracts.ncl" in
let profile = import "default.ncl" in
(profile & { ambient_tool_path = "/usr/bin/verilator" })
  | Contracts.HardwareProfile
  | Contracts.ExactHardwareProfile
