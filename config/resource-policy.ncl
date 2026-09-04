# Reviewable defaults for evidence-driven remote resource policy.
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Positive = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let BasisPoints = std.contract.from_predicate (fun value =>
  std.is_number value && value >= 10000 && value <= 40000
) in
let SelectionPolicy = {
  schema | std.contract.Equal "mantle-resource-selection-policy-v1",
  policy_id | NonEmpty,
  policy_version | NonEmpty,
  sample_count_min | Positive,
  observation_age_secs_max | Positive,
  memory_margin_basis_points | BasisPoints,
  scratch_margin_basis_points | BasisPoints,
  measurement_bytes_max | Positive,
  fallback | [| 'static-on-insufficient-history, 'reject-on-insufficient-history |],
} in
let RetryPolicy = {
  schema | std.contract.Equal "mantle-resource-retry-policy-v1",
  policy_id | NonEmpty,
  retry_count_max | Positive,
  class_ordinal_max | Positive,
  cumulative_charge_units_max | Positive,
  wall_time_ms_max | Positive,
} in
let UnitSchedule = {
  schedule_id | NonEmpty,
  cpu_ms_per_unit | Positive,
  memory_byte_ms_per_unit | Positive,
  transfer_bytes_per_unit | Positive,
  charge_units_max | Positive,
} in
let Rollout = {
  mode | [| 'observe-only, 'enforce |],
  project_opted_in | Bool,
  historical_selection_enabled | Bool,
  oom_retry_enabled | Bool,
  quota_enforcement_enabled | Bool,
  result_sharing_enabled | Bool,
} in
let Reference = {
  repository | std.contract.Equal "github.com/nixbuild/nixbench",
  revision | NonEmpty,
  source_blake3 | Blake3,
  license | std.contract.Equal "Apache-2.0",
  license_blake3 | Blake3,
  adaptation | NonEmpty,
} in
let Profile = {
  schema | std.contract.Equal "mantle-resource-policy-profile-v1",
  selection | SelectionPolicy,
  retry | RetryPolicy,
  accounting | UnitSchedule,
  rollout | Rollout,
  benchmark_reference | Reference,
  valence_profile | std.contract.Equal "valence.build-service-evidence.v1",
  valence_revision | NonEmpty,
  non_claim | NonEmpty,
} in

let seconds_per_day = 86400 in
let observation_age_days_max = 30 in
let basis_points_one = 10000 in
let default_memory_margin_basis_points = 12500 in
let default_scratch_margin_basis_points = 11000 in
let default_measurement_bytes_max = 1125899906842624 in
let default_retry_count_max = 2 in
let default_class_ordinal_max = 1024 in
let retry_charge_units_max = 1000000 in
let retry_wall_time_ms_max = 7200000 in
let unit_divisor = 1000 in
let ledger_charge_units_max = 1000000000000 in

{
  schema = "mantle-resource-policy-profile-v1",
  selection = {
    schema = "mantle-resource-selection-policy-v1",
    policy_id = "mantle-resource-selection-default",
    policy_version = "v1",
    sample_count_min = default_retry_count_max,
    observation_age_secs_max = seconds_per_day * observation_age_days_max,
    memory_margin_basis_points = default_memory_margin_basis_points,
    scratch_margin_basis_points = default_scratch_margin_basis_points,
    measurement_bytes_max = default_measurement_bytes_max,
    fallback = 'static-on-insufficient-history,
  },
  retry = {
    schema = "mantle-resource-retry-policy-v1",
    policy_id = "mantle-resource-retry-default",
    retry_count_max = default_retry_count_max,
    class_ordinal_max = default_class_ordinal_max,
    cumulative_charge_units_max = retry_charge_units_max,
    wall_time_ms_max = retry_wall_time_ms_max,
  },
  accounting = {
    schedule_id = "mantle-public-resource-units-v1",
    cpu_ms_per_unit = unit_divisor,
    memory_byte_ms_per_unit = unit_divisor,
    transfer_bytes_per_unit = unit_divisor,
    charge_units_max = ledger_charge_units_max,
  },
  rollout = {
    mode = 'observe-only,
    project_opted_in = false,
    historical_selection_enabled = false,
    oom_retry_enabled = false,
    quota_enforcement_enabled = false,
    result_sharing_enabled = false,
  },
  benchmark_reference = {
    repository = "github.com/nixbuild/nixbench",
    revision = "b256cd275d8c79ba485be8d317005f973879825a",
    source_blake3 = "737a5a42181fdd59ba451159db7558c516f31ea82ffcc771f66a0f2d6421a5a8",
    license = "Apache-2.0",
    license_blake3 = "14ec1590aae4c4e763d123c31085d5d37703f7a838e589b0d356058ad99177dd",
    adaptation = "fixed-seed-size-compressibility-and-cpu-shape-only",
  },
  valence_profile = "valence.build-service-evidence.v1",
  valence_revision = "e40c76b4d2070a29636e00c85c0dff93f03dba2f",
  non_claim = "resource policy records bounded decisions without proving future sufficiency, build correctness, host isolation, fair billing, or production performance",
} | Profile
