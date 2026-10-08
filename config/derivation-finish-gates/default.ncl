let Contracts = import "contracts.ncl" in
{
  schema = "mantle-derivation-finish-policy-v1",
  max_command_args = 16,
  max_command_arg_bytes = 4096,
  max_scan_hits = 128,
  max_excerpt_bytes = 160,
  max_optional_sonames = 32,
  gates = {},
} | Contracts.Policy
