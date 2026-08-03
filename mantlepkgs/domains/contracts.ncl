# Typed review contracts for Mantlepkgs domain composition.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let ExactRevision = std.contract.from_predicate (fun value =>
  std.is_string value
  && (std.string.is_match "^[0-9a-f]{40}$" value || std.string.is_match "^[0-9a-f]{64}$" value)
) in
let PositiveInteger = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value > 0
) in
let DomainClass = std.contract.from_predicate (fun value =>
  value == "core" || value == "ecosystem"
) in
let Selector = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "(^\\.|\\.$|\\.\\.|/|\\\\)" value
) in
let SourceLockContract = {
  repository | NonEmptyString,
  reference | NonEmptyString,
  revision | ExactRevision,
  source_digest_blake3 | Blake3,
} in
let ArtifactContract = {
  role | NonEmptyString,
  digest_blake3 | Blake3,
} in
let PackageContract = {
  package_identity_blake3 | Blake3,
  name | NonEmptyString,
  public_selector | Selector,
  system | NonEmptyString,
  root_identity_blake3 | Blake3,
  policy_digest_blake3 | Blake3,
  buildable | Bool,
  aliases | Array Selector,
  artifacts | Array ArtifactContract,
} in
let ShardLimitsContract = {
  max_packages | PositiveInteger,
  max_aliases | PositiveInteger,
  max_artifacts | PositiveInteger,
} in
let ShardContract = {
  schema | std.contract.from_predicate (fun value => value == "mantlepkgs-domain-shard-v1"),
  shard_identity_blake3 | Blake3,
  name | NonEmptyString,
  class | DomainClass,
  owner_label | NonEmptyString,
  source | SourceLockContract,
  source_catalog_identity_blake3 | Blake3,
  packages | Array PackageContract,
  limits | ShardLimitsContract,
} in
let VariantContract = {
  variant_identity_blake3 | Blake3,
  public_selector | Selector,
  system | NonEmptyString,
  base_selector | Selector,
  base_package_identity_blake3 | Blake3,
  base_root_identity_blake3 | Blake3,
  variant_name | NonEmptyString,
  changed_policy_digest_blake3 | Blake3,
  root_identity_blake3 | Blake3,
  provenance_digest_blake3 | Blake3,
  aliases | Array Selector,
} in
let ValidationLimitsContract = {
  max_sources | PositiveInteger,
  max_tools | PositiveInteger,
  max_dependencies | PositiveInteger,
  max_output_bytes | PositiveInteger,
  timeout_seconds | PositiveInteger,
} in
let ValidationRootContract = {
  validation_root_identity_blake3 | Blake3,
  name | NonEmptyString,
  system | NonEmptyString,
  package_selector | Selector,
  package_identity_blake3 | Blake3,
  validation_selector | Selector,
  validation_package_identity_blake3 | Blake3,
  sources | Array ArtifactContract,
  tools | Array ArtifactContract,
  dependencies | Array ArtifactContract,
  policy_digest_blake3 | Blake3,
  expected_outcome | std.contract.from_predicate (fun value => value == "pass"),
  limits | ValidationLimitsContract,
} in
let CatalogLimitsContract = {
  max_shards | PositiveInteger,
  max_packages | PositiveInteger,
  max_aliases | PositiveInteger,
  max_variants | PositiveInteger,
  max_validation_roots | PositiveInteger,
  max_artifacts | PositiveInteger,
} in
let ManifestContract = {
  schema | std.contract.from_predicate (fun value => value == "mantlepkgs-domain-manifest-v1"),
  name | NonEmptyString,
  shards | Array ShardContract,
  variants | Array VariantContract,
  validation_roots | Array ValidationRootContract,
  limits | CatalogLimitsContract,
} in
{
  Manifest = ManifestContract,
  Shard = ShardContract,
  Package = PackageContract,
  Variant = VariantContract,
  ValidationRoot = ValidationRootContract,
  Artifact = ArtifactContract,
  SourceLock = SourceLockContract,
  CatalogLimits = CatalogLimitsContract,
}
