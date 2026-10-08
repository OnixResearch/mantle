let Contracts = import "contracts.ncl" in
{
  schema = "mantle-authority-caveats-v1",
  accepted_variants = ["rewrite", "reject", "alternatives", "unknown-deny-all"],
  limits = {
    max_chain = 8,
    max_alternatives = 8,
    max_pattern_bytes = 256,
    max_caveat_bytes = 8192,
    max_value_bytes = 512,
    max_segments = 16,
  },
  order = "newest-first-right-to-left",
  unknown_behavior = "deny-every-input",
  rejected_message_behavior = "silent-discard",
  original_request_policy_recheck = true,
  parent_prefix_policy_recheck = true,
  non_claims = [
    "filter-is-not-authentication-or-token-verification",
    "filter-is-not-key-resolution-or-revocation-freshness",
    "filter-is-not-build-correctness-or-release-eligibility",
  ],
} | Contracts.Policy
