let Contracts = import "../contracts.ncl" in
{
  policy_id = "mantle-store-overlay-trust-v2",
  verification = "ed25519-pathinfo",
  unknown_policy = "allow",
  shadow_failure = "block",
  skip_failure_classes = [],
} | Contracts.Trust
