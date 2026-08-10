let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-version-selection-policy-v1",
  policy_identity_blake3 = "",
  method = "newest-published-revision-for-reported-version",
  system = "x86_64-linux",
  # The resolve shell binds these placeholders to the explicit --index input.
  accepted_cohort_identity_blake3 = "",
  accepted_observation_set_identity_blake3 = "",
  accepted_index_identity_blake3 = "",
  limits = {
    max_revisions = 2,
    max_attributes = 1,
    max_observations = 2,
    max_requests = 2,
    max_groups = 2,
    max_diagnostics = 16,
    max_aliases_per_request = 2,
    max_text_bytes = 256,
  },
} | Contracts.Policy
