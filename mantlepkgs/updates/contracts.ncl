# Typed contracts for deterministic Mantlepkgs source-update plans.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonExecutableString = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "\\$\\(|`|\\$\\{|;|[\\n\\r]" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Blake3OrPlaceholder = std.contract.from_predicate (fun value =>
  value == "" || (std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value)
) in
let PositiveInteger = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value > 0
) in
let NonNegativeInteger = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value >= 0
) in
let SafeRelativePath = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "^/|/$|//|(^|/)\\.\\.?(/|$)|\\\\" value
) in
let JsonPointer = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^/[^/]+(/[^/]+)*$" value
) in
let HttpsAuthority = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^https://[^/@?#[:space:]]+(:[0-9]+)?$" value
) in
let HttpsUrl = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^https://[^/@?#[:space:]]+[^@[:space:]]*$" value
) in
let SourceQuery = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^/[^/#\\\\[:space:]][^#\\\\[:space:]]*$" value
) in
let SourceKind = std.contract.from_predicate (fun value =>
  std.array.elem value ["git-tags", "release-index", "directory-index", "ecosystem-registry"]
) in
let ObservationStatus = std.contract.from_predicate (fun value =>
  std.array.elem value ["success", "unavailable", "failed"]
) in
let AdvisoryService = std.contract.from_predicate (fun value =>
  std.array.elem value ["osv", "repology"]
) in
let AdvisoryMode = std.contract.from_predicate (fun value =>
  std.array.elem value ["disabled", "optional", "required"]
) in
let EvidenceStatus = std.contract.from_predicate (fun value =>
  std.array.elem value ["success", "missing", "unavailable", "failed"]
) in
let ProposalStatus = std.contract.from_predicate (fun value =>
  std.array.elem value ["ready-for-review", "blocked", "unavailable", "failed", "no-candidate"]
) in
let PackageKey = {
  public_selector | NonEmptyString,
} in
let Migration = {
  source_schema | Dyn,
  source_policy_identity_blake3 | Dyn,
  reviewed | Bool,
} in
let VersionRules = {
  required_prefix | String,
  component_count | PositiveInteger,
  max_component | PositiveInteger,
  minimum_version | Dyn,
  maximum_version | Dyn,
  ignored_versions | Array NonEmptyString,
  allow_prerelease | Bool,
  odd_minor_is_development | Bool,
  high_patch_development_from | Dyn,
} in
let PatchPolicy = {
  allow_major | Bool,
  allow_minor | Bool,
  allow_patch | Bool,
} in
let AdvisoryRequirement = {
  mode | AdvisoryMode,
  service_identity | String,
  query | HttpsUrl,
  package_coordinate | String,
} in
let AdvisoryPolicy = {
  osv | AdvisoryRequirement,
  repology | AdvisoryRequirement,
  block_on_findings | Bool,
} in
let ValidationPolicy = {
  require_catalog | Bool,
  require_build_observations | Bool,
  require_validation_roots | Bool,
  require_impact_report | Bool,
} in
let MutationTarget = {
  relative_path | SafeRelativePath,
  version_pointer | JsonPointer,
  source_ref_pointer | JsonPointer,
  source_identity_pointer | JsonPointer,
} in
let Limits = {
  max_response_bytes | PositiveInteger,
  max_document_bytes | PositiveInteger,
  max_plan_bytes | PositiveInteger,
  max_candidates | PositiveInteger,
  max_findings | PositiveInteger,
  max_effects | PositiveInteger,
  max_diagnostics | PositiveInteger,
  max_artifacts | PositiveInteger,
  max_redirects | NonNegativeInteger,
  max_retries | NonNegativeInteger,
  max_elapsed_millis | PositiveInteger,
} in
let PolicyContract = {
  schema | std.contract.Equal "mantlepkgs-update-policy-v1",
  policy_identity_blake3 | Blake3OrPlaceholder,
  migration | Migration,
  package | PackageKey,
  system | NonEmptyString,
  source_kind | SourceKind,
  adapter_identity | NonEmptyString,
  source_authority | HttpsAuthority,
  source_query | SourceQuery,
  response_schema | NonEmptyString,
  current_version | NonEmptyString,
  current_source_ref | NonEmptyString,
  current_source_identity_blake3 | Blake3,
  version_rules | VersionRules,
  patch_policy | PatchPolicy,
  advisory_policy | AdvisoryPolicy,
  validation_policy | ValidationPolicy,
  mutation | MutationTarget,
  allow_ambient_credentials | std.contract.Equal false,
  allow_ambient_proxy | std.contract.Equal false,
  limits | Limits,
} in
let SourceCandidate = {
  version | NonEmptyString,
  source_ref | NonEmptyString,
  source_identity_blake3 | Blake3,
} in
let CollectionFacts = {
  response_bytes | NonNegativeInteger,
  redirect_count | NonNegativeInteger,
  retry_count | NonNegativeInteger,
  elapsed_millis | NonNegativeInteger,
} in
let SourceResponseContract = {
  schema | NonEmptyString,
  candidates | Array SourceCandidate,
} in
let SourceObservationContract = {
  schema | std.contract.Equal "mantlepkgs-source-observation-v1",
  observation_identity_blake3 | Blake3OrPlaceholder,
  policy_identity_blake3 | Blake3,
  adapter_identity | NonEmptyString,
  source_kind | SourceKind,
  query | NonEmptyString,
  source_authority | HttpsAuthority,
  response_schema | NonEmptyString,
  response_identity_blake3 | Dyn,
  status | ObservationStatus,
  candidates | Array SourceCandidate,
  reason_codes | Array NonEmptyString,
  collection | CollectionFacts,
} in
let AdvisoryFinding = {
  finding_id | NonEmptyString,
  finding_identity_blake3 | Blake3,
} in
let AdvisoryResponseContract = {
  schema | NonEmptyString,
  package_coordinate | NonEmptyString,
  version | NonEmptyString,
  findings | Array AdvisoryFinding,
} in
let AdvisoryObservationContract = {
  schema | std.contract.Equal "mantlepkgs-advisory-observation-v1",
  observation_identity_blake3 | Blake3OrPlaceholder,
  policy_identity_blake3 | Blake3,
  service | AdvisoryService,
  service_identity | NonEmptyString,
  query | NonEmptyString,
  package_coordinate | NonEmptyString,
  version | NonEmptyString,
  response_schema | NonEmptyString,
  response_identity_blake3 | Dyn,
  status | ObservationStatus,
  findings | Array AdvisoryFinding,
  reason_codes | Array NonEmptyString,
  collection | CollectionFacts,
} in
let LinkedEvidence = {
  status | EvidenceStatus,
  artifact_identity_blake3 | Array Blake3,
  reason_codes | Array NonEmptyString,
} in
let ValidationEvidenceContract = {
  schema | std.contract.Equal "mantlepkgs-update-validation-evidence-v1",
  candidate_version | NonEmptyString,
  candidate_source_identity_blake3 | Blake3,
  catalog | LinkedEvidence,
  build_observations | LinkedEvidence,
  validation_roots | LinkedEvidence,
  impact_report | LinkedEvidence,
} in
let StructuredEdit = {
  field_pointer | JsonPointer,
  old_value | Dyn,
  new_value | Dyn,
  reason_code | NonEmptyString,
} in
let MutationEffect = {
  relative_path | SafeRelativePath,
  input_digest_blake3 | Blake3,
  output_digest_blake3 | Blake3,
  edits | Array StructuredEdit,
  output_document | Dyn,
} in
let CandidateDecision = {
  candidate | SourceCandidate,
  normalized_version | Dyn,
  eligible | Bool,
  reason_codes | Array NonEmptyString,
} in
let Selection = {
  selected | Dyn,
  decisions | Array CandidateDecision,
  reason_codes | Array NonEmptyString,
} in
let PlanContract = {
  schema | std.contract.Equal "mantlepkgs-update-plan-v1",
  plan_identity_blake3 | Blake3,
  policy_identity_blake3 | Blake3,
  source_observation_identity_blake3 | Blake3,
  advisory_observation_identity_blake3 | Array Blake3,
  package | PackageKey,
  selection | Selection,
  effects | Array MutationEffect,
  validation_evidence | ValidationEvidenceContract,
  proposal_status | ProposalStatus,
  reason_codes | Array NonEmptyString,
  non_claims | Array NonEmptyString,
} in
{
  Policy = PolicyContract,
  SourceResponse = SourceResponseContract,
  SourceObservation = SourceObservationContract,
  AdvisoryResponse = AdvisoryResponseContract,
  AdvisoryObservation = AdvisoryObservationContract,
  ValidationEvidence = ValidationEvidenceContract,
  Plan = PlanContract,
}
