# Reviewable demonstration trust policy.
# Replace the public key and trust domain for production. The matching private
# key is intentionally not checked into this repository.
let trust = import "oci_registry_trust.ncl" in
({
  schema = "mantle-oci-registry-trust-policy-v1",
  schema_version = 1,
  trust_domain = "onix-kernel-bundle",
  allowed_repositories = ["onix/kernel-bundle"],
  trusted_public_keys = [
    "cache.example.com-1:yKUSiqP9yaMSduDmGtw8U9iVVd/Coyv9csB1rjHtiRM=",
  ],
  required_signers = ["cache.example.com-1"],
  minimum_signatures = 1,
  revoked_public_key_blake3 = [],
} | trust.RegistryTrustPolicy)
