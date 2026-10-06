let Contracts = import "../contracts.ncl" in
let good = (import "../default.ncl").probe in
{
  invocation = good.invocation,
  script_identity = good.script_identity,
  toolchain_identity = good.toolchain_identity,
  dependency_identity = good.dependency_identity,
  platform_identity = good.platform_identity,
  flags_identity = good.flags_identity,
  environment = good.environment,
  failure_replay = good.failure_replay,
  max_diagnostic_bytes = 262145,
} | Contracts.ProbePolicy
