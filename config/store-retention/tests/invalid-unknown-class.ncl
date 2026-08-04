let Contracts = import "../contracts.ncl" in
{
  durability = "unknown",
  eligible_owner_scopes = ["operator"],
  explicit_removal_required = true,
} | Contracts.RootRule
