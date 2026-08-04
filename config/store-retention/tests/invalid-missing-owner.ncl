let Contracts = import "../contracts.ncl" in
{
  durability = "durable",
  explicit_removal_required = true,
} | Contracts.RootRule
