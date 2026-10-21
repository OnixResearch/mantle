# Mantle stdlib: typed OCI registry signature trust policy.
#
# Rust performs bounded semantic validation and Ed25519 verification after
# Nickel checks this reviewable policy shape.
#
# Nickel record contracts may carry at most one field without a definition, so
# the policy shape is a record type: every policy field is required.

let RegistryTrustPolicyContract = {
  schema : String,
  schema_version : Number,
  trust_domain : String,
  allowed_repositories : Array String,
  trusted_public_keys : Array String,
  required_signers : Array String,
  minimum_signatures : Number,
  revoked_public_key_blake3 : Array String,
} in

{
  RegistryTrustPolicy = RegistryTrustPolicyContract,
}
