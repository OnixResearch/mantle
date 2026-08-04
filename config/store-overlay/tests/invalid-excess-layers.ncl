let Contracts = import "../contracts.ncl" in
{
  max_base_layers = 33,
  max_generation_members = 1,
  max_generation_bytes = 1,
  max_descriptor_bytes = 1,
} | Contracts.Limits
