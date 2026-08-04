let Contracts = import "../contracts.ncl" in
{
  require_owner = true,
  require_identity = true,
  reject_clock_rollback = true,
  unsafe_state = "",
} | Contracts.LeasePolicy
