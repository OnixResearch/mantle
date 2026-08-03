# Typed review contract for one Mantlepkgs generation manifest.
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
let SupportedSystem = std.contract.from_predicate (fun value => value == "x86_64-linux") in
let RelativePath = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^[^/].*" value
  && !std.string.is_match "(^|/)\\.\\.?(/|$)" value
  && !std.string.is_match "//" value
) in
let AbsoluteStorePrefix = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^/[^/]+(/[^/]+)*$" value
  && !std.string.is_match "(^|/)\\.\\.?(/|$)" value
) in
let SourceLockContract = {
  reference | NonEmptyString,
  revision | ExactRevision,
  lock_digest_blake3 | Blake3,
} in
let SelectorContract = {
  name | NonEmptyString,
  attribute | NonEmptyString,
  system | SupportedSystem,
  aliases | Array NonEmptyString,
} in
let ArtifactContract = {
  path | RelativePath,
  digest_blake3 | Blake3,
} in
let ConversionPolicyContract = {
  mode | std.contract.from_predicate (fun value => value == "recompute-blake3-v1"),
  target_store_prefix | AbsoluteStorePrefix,
  translation_policy | ArtifactContract,
  execution_profile | ArtifactContract,
} in
let SourcePolicyContract = {
  mode | std.contract.from_predicate (fun value => value == "source-bundle-required-v1"),
  optional_transports | Array (std.contract.from_predicate (fun value => value == "nario-v2")),
} in
let OutputLayoutContract = {
  generation_directory | RelativePath,
} in
let LimitsContract = {
  max_selectors | PositiveInteger,
  max_graph_nodes | PositiveInteger,
  max_graph_bytes | PositiveInteger,
  max_source_requirements | PositiveInteger,
  max_artifact_bytes | PositiveInteger,
} in
let ManifestContract = {
  schema | std.contract.from_predicate (fun value => value == "mantlepkgs-manifest-v1"),
  source | SourceLockContract,
  systems | Array SupportedSystem,
  selectors | Array SelectorContract,
  conversion_policy | ConversionPolicyContract,
  source_policy | SourcePolicyContract,
  output | OutputLayoutContract,
  limits | LimitsContract,
} in
{
  Manifest = ManifestContract,
  SourceLock = SourceLockContract,
  Selector = SelectorContract,
  ConversionPolicy = ConversionPolicyContract,
  SourcePolicy = SourcePolicyContract,
  OutputLayout = OutputLayoutContract,
  Limits = LimitsContract,
}
