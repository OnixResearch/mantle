# Typed review contracts for local Mantlepkgs impact inputs and reports.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
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
let AbsoluteStorePrefix = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^/[^/].*[^/]$" value
  && !std.string.is_match "//|/\\.\\.?(/|$)" value
) in
let Selector = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "(^\\.|\\.$|\\.\\.|/|\\\\)" value
) in
let PackageKind = std.contract.from_predicate (fun value =>
  value == "package" || value == "variant"
) in
let ObservationState = std.contract.from_predicate (fun value =>
  std.array.elem value ["success", "failure", "blocked", "not-attempted", "unavailable"]
) in
let CatalogDisposition = std.contract.from_predicate (fun value =>
  std.array.elem value [
    "added",
    "removed",
    "unchanged",
    "recipe-changed",
    "policy-changed",
    "blocker-changed",
    "variant-changed",
  ]
) in
let ChangedDimension = std.contract.from_predicate (fun value =>
  std.array.elem value ["recipe", "policy", "blocker", "variant"]
) in
let TransitionDisposition = std.contract.from_predicate (fun value =>
  std.array.elem value [
    "newly-successful",
    "still-successful",
    "newly-failed",
    "still-failed",
    "blocked",
    "not-attempted",
    "unavailable",
    "missing",
  ]
) in
let DeltaDirection = std.contract.from_predicate (fun value =>
  std.array.elem value ["increase", "decrease", "unchanged"]
) in
let PackageKey = {
  public_selector | Selector,
} in
let Blocker = {
  code | NonEmptyString,
  detail_identity_blake3 | Blake3,
} in
let VariantFacts = {
  base_package_identity_blake3 | Blake3,
  base_root_identity_blake3 | Blake3,
  variant_name | NonEmptyString,
  provenance_identity_blake3 | Blake3,
} in
let PackageRecordContract = {
  schema | std.contract.Equal "mantlepkgs-impact-package-record-v1",
  record_identity_blake3 | Blake3OrPlaceholder,
  key | PackageKey,
  kind | PackageKind,
  package_identity_blake3 | Blake3,
  root_identity_blake3 | Blake3,
  recipe_identity_blake3 | Blake3,
  policy_identity_blake3 | Blake3,
  is_buildable | Bool,
  blockers | Array Blocker,
  variant | VariantFacts | optional,
} in
let Authority = std.contract.from_validator (fun value =>
  if std.is_record value && std.record.has_field "kind" value then
    let kind = value.kind in
    if kind == "admitted-action-result" then
      if std.record.has_field "runtime_report_schema" value
        && std.record.has_field "runtime_report_identity_blake3" value
        && std.record.has_field "action_ref" value
        && std.record.has_field "selected_result_ref" value
        && std.record.has_field "request_identity_blake3" value
        && std.record.has_field "policy_identity_blake3" value
        && std.record.has_field "platform_identity_blake3" value
        && std.record.has_field "signature_identity_blake3" value
        && std.record.has_field "output_set_identity_blake3" value
        && std.record.has_field "cas_identity_blake3" value
      then 'Ok
      else 'Error { message = "admitted action-result authority is incomplete" }
    else if kind == "realization-receipt" then
      if std.record.has_field "receipt_schema" value
        && std.record.has_field "receipt_identity_blake3" value
      then 'Ok
      else 'Error { message = "realization authority is incomplete" }
    else if kind == "explicit-status" then
      if std.record.has_field "status_schema" value
        && std.record.has_field "status_identity_blake3" value
      then 'Ok
      else 'Error { message = "explicit status authority is incomplete" }
    else 'Error { message = "unknown observation authority" }
  else 'Error { message = "observation authority must be a tagged record" }
) in
let ObservationContract = {
  schema | std.contract.Equal "mantlepkgs-impact-observation-v1",
  observation_identity_blake3 | Blake3OrPlaceholder,
  key | PackageKey,
  package_record_identity_blake3 | Blake3,
  state | ObservationState,
  terminal_reason_codes | Array NonEmptyString,
  authority | Authority,
} in
let ClosureMember = {
  member_identity_blake3 | Blake3,
  store_path | NonEmptyString,
  path_info_identity_blake3 | Blake3 | optional,
  logical_bytes | PositiveInteger | optional,
  references | Array Blake3,
} in
let ClosureContract = {
  schema | std.contract.Equal "mantlepkgs-impact-closure-v1",
  closure_identity_blake3 | Blake3OrPlaceholder,
  key | PackageKey,
  package_record_identity_blake3 | Blake3,
  root_member_identity_blake3 | Blake3,
  system | NonEmptyString,
  store_prefix | AbsoluteStorePrefix,
  closure_semantics | NonEmptyString,
  byte_semantics | NonEmptyString,
  is_complete | Bool,
  incomplete_reason_codes | Array NonEmptyString,
  members | Array ClosureMember,
} in
let PolicyContract = {
  schema | std.contract.Equal "mantlepkgs-impact-policy-v1",
  policy_identity_blake3 | Blake3OrPlaceholder,
  max_packages | PositiveInteger,
  max_variants | PositiveInteger,
  max_closure_members | PositiveInteger,
  max_dependency_edges | PositiveInteger,
  max_observations | PositiveInteger,
  max_diagnostics | PositiveInteger,
  max_report_bytes | PositiveInteger,
} in
let SnapshotContract = {
  schema | std.contract.Equal "mantlepkgs-impact-snapshot-v1",
  snapshot_identity_blake3 | Blake3OrPlaceholder,
  comparison_policy_identity_blake3 | Blake3OrPlaceholder,
  catalog_schema | NonEmptyString,
  catalog_identity_blake3 | Blake3,
  system | NonEmptyString,
  store_prefix | AbsoluteStorePrefix,
  conversion_policy_identity_blake3 | Blake3,
  package_record_schema | std.contract.Equal "mantlepkgs-impact-package-record-v1",
  observation_schema | std.contract.Equal "mantlepkgs-impact-observation-v1",
  identity_domain | std.contract.Equal "mantle.mantlepkgs.impact-record.v1",
  packages | Array PackageRecordContract,
  observations | Array ObservationContract,
  closures | Array ClosureContract,
} in
let LogicalByteDelta = {
  direction | DeltaDirection,
  magnitude_bytes | PositiveInteger,
} in
let BuildTransition = {
  disposition | TransitionDisposition,
  base_state | Dyn,
  head_state | Dyn,
  base_observation_identity_blake3 | Dyn,
  head_observation_identity_blake3 | Dyn,
  base_terminal_reason_codes | Array String,
  head_terminal_reason_codes | Array String,
} in
let ClosureDelta = {
  is_comparable | Bool,
  reason_codes | Array String,
  base_closure_identity_blake3 | Dyn,
  head_closure_identity_blake3 | Dyn,
  added_member_identities | Dyn,
  removed_member_identities | Dyn,
  retained_dependency_identities | Dyn,
  member_count_delta | Dyn,
  logical_byte_delta | Dyn,
} in
let PackageImpact = {
  key | PackageKey,
  catalog_disposition | CatalogDisposition,
  changed_dimensions | Array ChangedDimension,
  base_record_identity_blake3 | Dyn,
  head_record_identity_blake3 | Dyn,
  build_transition | BuildTransition,
  closure_delta | ClosureDelta,
} in
let ReportContract = {
  schema | std.contract.Equal "mantle-package-impact-v1",
  report_identity_blake3 | Blake3,
  comparison_policy_identity_blake3 | Blake3,
  base_snapshot_identity_blake3 | Blake3,
  head_snapshot_identity_blake3 | Blake3,
  base_catalog_identity_blake3 | Blake3,
  head_catalog_identity_blake3 | Blake3,
  system | NonEmptyString,
  store_prefix | AbsoluteStorePrefix,
  conversion_policy_identity_blake3 | Blake3,
  package_impacts | Array PackageImpact,
  non_claims | Array NonEmptyString,
} in
{
  Policy = PolicyContract,
  Snapshot = SnapshotContract,
  PackageRecord = PackageRecordContract,
  Observation = ObservationContract,
  Closure = ClosureContract,
  Report = ReportContract,
}
