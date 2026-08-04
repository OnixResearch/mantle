let Contracts = import "../contracts.ncl" in
{
  policy_id = "mantle-store-overlay-trust-v2",
  verification = "signature-presence-only",
  unknown_policy = "reject",
  shadow_failure = "block",
  skip_failure_classes = [],
} | Contracts.Trust
