# Typed review contracts for Mantlepkgs historical version resolution.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3 = std.contract.from_predicate (fun value =>
  value == "" || (std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value)
) in
let RequiredBlake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let OptionalBlake3 = std.contract.from_predicate (fun value =>
  value == null || (std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value)
) in
let OptionalNonEmptyString = std.contract.from_predicate (fun value =>
  value == null || (std.is_string value && std.string.length value > 0)
) in
let OptionalRecord = std.contract.from_predicate (fun value =>
  value == null || std.is_record value
) in
let ExactRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let NarHash = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^sha256-[A-Za-z0-9+/]{43}=$" value
) in
let SupportedSystem = std.contract.from_predicate (fun value =>
  value == "x86_64-linux"
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
let Attribute = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.is_match "^[A-Za-z0-9_]+(\\.[A-Za-z0-9_]+)*$" value
) in
let Selector = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "(^\\.|\\.$|\\.\\.|/|\\\\)" value
) in
let ExactNixpkgsReference = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^github:NixOS/nixpkgs/[0-9a-f]{40}$" value
) in
let ObservationMethod = std.contract.from_predicate (fun value =>
  value == "package-version-attribute"
) in
let ObservationStatus = std.contract.from_predicate (fun value =>
  std.array.elem value ["success", "unavailable", "failed"]
) in
let ResolutionStatus = std.contract.from_predicate (fun value =>
  std.array.elem value ["resolved", "blocked"]
) in
let SelectionMethod = std.contract.from_predicate (fun value =>
  value == "newest-published-revision-for-reported-version"
) in
let LimitsContract = {
  max_revisions | PositiveInteger,
  max_attributes | PositiveInteger,
  max_observations | PositiveInteger,
  max_requests | PositiveInteger,
  max_groups | PositiveInteger,
  max_diagnostics | PositiveInteger,
  max_aliases_per_request | PositiveInteger,
  max_text_bytes | PositiveInteger,
} in
let CohortRevisionContract = {
  published_order | NonNegativeInteger,
  reference | ExactNixpkgsReference,
  revision | ExactRevision,
  nar_hash | NarHash,
} in
let CohortContract = {
  schema | std.contract.Equal "mantlepkgs-version-cohort-v1",
  cohort_identity_blake3 | Blake3,
  system | SupportedSystem,
  generator_identity | NonEmptyString,
  index_relative_path | SafeRelativePath,
  observation_method | ObservationMethod,
  revisions | Array CohortRevisionContract,
  attributes | Array Attribute,
  limits | LimitsContract,
} in
let ObservationContract = {
  schema | std.contract.Equal "mantlepkgs-version-observation-v1",
  observation_identity_blake3 | Blake3,
  cohort_identity_blake3 | RequiredBlake3,
  system | SupportedSystem,
  source_reference | ExactNixpkgsReference,
  revision | ExactRevision,
  published_order | NonNegativeInteger,
  nar_hash | NarHash,
  attribute | Attribute,
  method | ObservationMethod,
  status | ObservationStatus,
  reported_version | OptionalNonEmptyString,
  response_identity_blake3 | OptionalBlake3,
  reason_codes | Array NonEmptyString,
} in
let ObservationSetContract = {
  schema | std.contract.Equal "mantlepkgs-version-observation-set-v1",
  observation_set_identity_blake3 | RequiredBlake3,
  cohort_identity_blake3 | RequiredBlake3,
  observations | Array ObservationContract,
} in
let IndexEntryContract = {
  system | SupportedSystem,
  attribute | Attribute,
  reported_version | NonEmptyString,
  source_reference | ExactNixpkgsReference,
  revision | ExactRevision,
  published_order | NonNegativeInteger,
  nar_hash | NarHash,
  observation_identity_blake3 | RequiredBlake3,
} in
let IndexContract = {
  schema | std.contract.Equal "mantlepkgs-version-index-v1",
  index_identity_blake3 | RequiredBlake3,
  cohort_identity_blake3 | RequiredBlake3,
  observation_set_identity_blake3 | RequiredBlake3,
  generator_identity | NonEmptyString,
  system | SupportedSystem,
  method | ObservationMethod,
  entries | Array IndexEntryContract,
  successful_observations | NonNegativeInteger,
  unavailable_observations | NonNegativeInteger,
  failed_observations | NonNegativeInteger,
} in
let PolicyContract = {
  schema | std.contract.Equal "mantlepkgs-version-selection-policy-v1",
  policy_identity_blake3 | Blake3,
  method | SelectionMethod,
  system | SupportedSystem,
  accepted_cohort_identity_blake3 | Blake3,
  accepted_observation_set_identity_blake3 | Blake3,
  accepted_index_identity_blake3 | Blake3,
  limits | LimitsContract,
} in
let RequestContract = {
  request_identity_blake3 | Blake3,
  system | SupportedSystem,
  attribute | Attribute,
  reported_version | NonEmptyString,
  public_selector | Selector,
  aliases | Array Selector,
  unversioned_default | Bool,
} in
let RequestSetContract = {
  schema | std.contract.Equal "mantlepkgs-version-request-set-v1",
  request_set_identity_blake3 | Blake3,
  policy_identity_blake3 | Blake3,
  requests | Array RequestContract,
} in
let SelectedSourceContract = {
  source_reference | ExactNixpkgsReference,
  revision | ExactRevision,
  published_order | NonNegativeInteger,
  nar_hash | NarHash,
  observation_identity_blake3 | RequiredBlake3,
} in
let ResolutionReceiptContract = {
  schema | std.contract.Equal "mantlepkgs-version-resolution-v1",
  receipt_identity_blake3 | RequiredBlake3,
  request_identity_blake3 | RequiredBlake3,
  policy_identity_blake3 | RequiredBlake3,
  index_identity_blake3 | RequiredBlake3,
  observation_set_identity_blake3 | RequiredBlake3,
  cohort_identity_blake3 | RequiredBlake3,
  system | SupportedSystem,
  attribute | Attribute,
  reported_version | NonEmptyString,
  public_selector | Selector,
  aliases | Array Selector,
  unversioned_default | Bool,
  method | SelectionMethod,
  status | ResolutionStatus,
  selected | OptionalRecord,
  blocker_codes | Array NonEmptyString,
  non_claims | Array NonEmptyString,
} in
let ResolutionSetContract = {
  schema | std.contract.Equal "mantlepkgs-version-resolution-set-v1",
  resolution_set_identity_blake3 | RequiredBlake3,
  request_set_identity_blake3 | RequiredBlake3,
  policy_identity_blake3 | RequiredBlake3,
  index_identity_blake3 | RequiredBlake3,
  receipts | Array ResolutionReceiptContract,
} in
let GroupSelectorContract = {
  request_identity_blake3 | RequiredBlake3,
  receipt_identity_blake3 | RequiredBlake3,
  attribute | Attribute,
  reported_version | NonEmptyString,
  public_selector | Selector,
  aliases | Array Selector,
} in
let RevisionGroupContract = {
  group_identity_blake3 | RequiredBlake3,
  system | SupportedSystem,
  source_reference | ExactNixpkgsReference,
  revision | ExactRevision,
  published_order | NonNegativeInteger,
  nar_hash | NarHash,
  selectors | Array GroupSelectorContract,
} in
let ProductionPlanContract = {
  schema | std.contract.Equal "mantlepkgs-version-production-plan-v1",
  plan_identity_blake3 | RequiredBlake3,
  resolution_set_identity_blake3 | RequiredBlake3,
  groups | Array RevisionGroupContract,
  blocked_receipt_identity_blake3 | Array RequiredBlake3,
} in
let RecheckEntryContract = {
  receipt_identity_blake3 | RequiredBlake3,
  attribute | Attribute,
  expected_version | NonEmptyString,
  observed_version | OptionalNonEmptyString,
  status | ObservationStatus,
  reason_codes | Array NonEmptyString,
} in
let GroupRecheckContract = {
  schema | std.contract.Equal "mantlepkgs-version-recheck-v1",
  recheck_identity_blake3 | Blake3,
  group_identity_blake3 | RequiredBlake3,
  source_reference | ExactNixpkgsReference,
  revision | ExactRevision,
  nar_hash | NarHash,
  source_tree_blake3 | OptionalBlake3,
  entries | Array RecheckEntryContract,
  reason_codes | Array NonEmptyString,
} in
let RecheckSetContract = {
  schema | std.contract.Equal "mantlepkgs-version-recheck-set-v1",
  recheck_set_identity_blake3 | Blake3,
  plan_identity_blake3 | RequiredBlake3,
  rechecks | Array GroupRecheckContract,
} in
{
  Cohort = CohortContract,
  CohortRevision = CohortRevisionContract,
  Observation = ObservationContract,
  ObservationSet = ObservationSetContract,
  Index = IndexContract,
  Policy = PolicyContract,
  Request = RequestContract,
  RequestSet = RequestSetContract,
  ResolutionReceipt = ResolutionReceiptContract,
  ResolutionSet = ResolutionSetContract,
  RevisionGroup = RevisionGroupContract,
  ProductionPlan = ProductionPlanContract,
  GroupRecheck = GroupRecheckContract,
  RecheckSet = RecheckSetContract,
  Limits = LimitsContract,
}
