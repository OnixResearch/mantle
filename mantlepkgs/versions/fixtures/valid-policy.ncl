let contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-version-selection-policy-v1",
  policy_identity_blake3 = "",
  method = "newest-published-revision-for-reported-version",
  system = "x86_64-linux",
  accepted_cohort_identity_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  accepted_observation_set_identity_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  accepted_index_identity_blake3 = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  limits = {
    max_revisions = 8,
    max_attributes = 8,
    max_observations = 64,
    max_requests = 16,
    max_groups = 8,
    max_diagnostics = 64,
    max_aliases_per_request = 8,
    max_text_bytes = 256,
  },
} | contracts.Policy
