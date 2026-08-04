# Typed contracts for Mantle store retention.

let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in

let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in

let HashAlgorithm = std.contract.from_predicate (fun value =>
  std.is_string value && value == "BLAKE3"
) in

let Durability = std.contract.from_predicate (fun value =>
  std.is_string value &&
  (value == "durable" || value == "generation" || value == "lease" || value == "protected-unclassified")
) in

let RootRuleContract = {
  durability | Durability,
  eligible_owner_scopes | Array NonEmptyString,
  explicit_removal_required | Bool,
} in

{
  RootRule = RootRuleContract,

  Limits = {
    max_roots | PositiveNumber,
    max_decisions | PositiveNumber,
    max_usage_objects | PositiveNumber,
    max_roots_per_object | PositiveNumber,
    retained_project_output_generations | PositiveNumber,
    retained_project_source_generations | PositiveNumber,
    max_lease_renewals | PositiveNumber,
    default_lease_seconds | PositiveNumber,
    max_lease_seconds | PositiveNumber,
    max_closure_links_per_explanation | PositiveNumber,
  },

  Rules = {
    explicit_pin | RootRuleContract,
    project_output_generation | RootRuleContract,
    project_source_generation | RootRuleContract,
    active_shell_lease | RootRuleContract,
    bootstrap | RootRuleContract,
    self_build | RootRuleContract,
    remote_result | RootRuleContract,
    legacy_unmanaged | RootRuleContract,
  },

  LeasePolicy = {
    require_owner | Bool,
    require_identity | Bool,
    reject_clock_rollback | Bool,
    unsafe_state | NonEmptyString,
  },

  TransitionPolicy = {
    plan_is_non_mutating | Bool,
    require_explicit_execution | Bool,
    require_plan_identity | Bool,
    revalidate_bound_facts | Bool,
    partial_failure_is_incomplete | Bool,
  },

  Policy = {
    schema | NonEmptyString,
    policy_name | NonEmptyString,
    hash_algorithm | HashAlgorithm,
    limits | Limits,
    rules | Rules,
    leases | LeasePolicy,
    transitions | TransitionPolicy,
    non_claims | Array NonEmptyString,
  },
}
