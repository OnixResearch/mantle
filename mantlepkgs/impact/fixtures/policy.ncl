let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-impact-policy-v1",
  policy_identity_blake3 = "",
  max_packages = 256,
  max_variants = 64,
  max_closure_members = 4096,
  max_dependency_edges = 16384,
  max_observations = 256,
  max_diagnostics = 64,
  max_report_bytes = 16777216,
} | Contracts.Policy
