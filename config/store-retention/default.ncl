let Contracts = import "contracts.ncl" in
let MaxRoots = 65536 in
let MaxDecisions = 65536 in
let MaxUsageObjects = 1000000 in
let MaxRootsPerObject = 65536 in
let RetainedProjectOutputGenerations = 2 in
let RetainedProjectSourceGenerations = 2 in
let MaxLeaseRenewals = 1024 in
let DefaultLeaseSeconds = 28800 in
let MaxLeaseSeconds = 86400 in
let MaxClosureLinksPerExplanation = 256 in

let durable_rule = {
  durability = "durable",
  eligible_owner_scopes = ["operator", "project", "system"],
  explicit_removal_required = true,
} in

{
  schema = "mantle-store-retention-policy-v1",
  policy_name = "mantle-store-retention-default-v1",
  hash_algorithm = "BLAKE3",

  limits = {
    max_roots = MaxRoots,
    max_decisions = MaxDecisions,
    max_usage_objects = MaxUsageObjects,
    max_roots_per_object = MaxRootsPerObject,
    retained_project_output_generations = RetainedProjectOutputGenerations,
    retained_project_source_generations = RetainedProjectSourceGenerations,
    max_lease_renewals = MaxLeaseRenewals,
    default_lease_seconds = DefaultLeaseSeconds,
    max_lease_seconds = MaxLeaseSeconds,
    max_closure_links_per_explanation = MaxClosureLinksPerExplanation,
  },

  rules = {
    explicit_pin = durable_rule,
    project_output_generation = {
      durability = "generation",
      eligible_owner_scopes = ["project"],
      explicit_removal_required = true,
    },
    project_source_generation = {
      durability = "generation",
      eligible_owner_scopes = ["project"],
      explicit_removal_required = true,
    },
    active_shell_lease = {
      durability = "lease",
      eligible_owner_scopes = ["operator", "project"],
      explicit_removal_required = true,
    },
    bootstrap = durable_rule,
    self_build = durable_rule,
    remote_result = {
      durability = "generation",
      eligible_owner_scopes = ["remote", "project"],
      explicit_removal_required = true,
    },
    legacy_unmanaged = {
      durability = "protected-unclassified",
      eligible_owner_scopes = ["legacy-unmanaged"],
      explicit_removal_required = true,
    },
  },

  leases = {
    require_owner = true,
    require_identity = true,
    reject_clock_rollback = true,
    unsafe_state = "quarantine",
  },

  transitions = {
    plan_is_non_mutating = true,
    require_explicit_execution = true,
    require_plan_identity = true,
    revalidate_bound_facts = true,
    partial_failure_is_incomplete = true,
  },

  non_claims = [
    "retention-does-not-prove-content-correctness",
    "retention-does-not-prove-rebuildability",
    "usage-does-not-prove-exclusive-ownership",
    "plan-does-not-prove-successful-deletion",
  ],
} | Contracts.Policy
