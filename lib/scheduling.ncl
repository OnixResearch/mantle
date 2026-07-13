# Deterministic lazy ready-goal scheduling policy.
#
# This contract is provider-neutral. Runtime shells may supply immutable policy
# and history snapshots, while the Rust scheduling core validates the same
# schema and rejects malformed facts before mutating the ready set.
#
# r[impl build_scheduling.deterministic_priority_kernel]
# r[impl build_scheduling.deterministic_starvation_bound]

let DefaultAgedAfterEpochs = 4 in
let DefaultProtectedAfterEpochs = 8 in
let DefaultMaxKnownGoals = 10000 in
let DefaultMaxKnownEdges = 640000 in
let DefaultMaxBlockedRootPressure = 256 in
let DefaultMaxHistoryEntries = 10000 in
let DefaultMaxHistoryKeyBytesTotal = 4194304 in
let DefaultMaxHistorySnapshotBytes = 8388608 in
let DefaultMaxResourceClasses = 32 in
let DefaultMaxResourceClassNameBytes = 64 in
let DefaultMaxResourceLeases = 4096 in
let DefaultMaxLocalityObservations = 4096 in
let DefaultMaxCpuUnits = 1048576 in
let DefaultMaxResourceCount = 1048576 in
let DefaultMaxResourceBytes = 1125899906842624 in
let DefaultMaxPlacementCandidates = 1024 in

let PreferenceFieldContract = [|
  'known-graph,
  'resource-fit,
  'locality-transfer,
|] in

let HistoryFallbackContract = [|
  'structural,
|] in

let AgePolicyContract = {
  aged_after_epochs
    | doc "Scheduling epochs before an ordinary ready goal enters the aged class."
    | Number
    | default = DefaultAgedAfterEpochs,
  protected_after_epochs
    | doc "Scheduling epochs before an eligible ready goal outranks ordinary graph/resource/locality preference."
    | Number
    | default = DefaultProtectedAfterEpochs,
} in

let KnownGraphBoundsContract = {
  max_goals
    | Number
    | default = DefaultMaxKnownGoals,
  max_edges
    | Number
    | default = DefaultMaxKnownEdges,
  max_blocked_root_pressure
    | Number
    | default = DefaultMaxBlockedRootPressure,
  basis
    | [| 'known-graph |]
    | default = 'known-graph,
} in

let HistoryPolicyContract = {
  max_entries
    | Number
    | default = DefaultMaxHistoryEntries,
  max_key_bytes_total
    | Number
    | default = DefaultMaxHistoryKeyBytesTotal,
  max_snapshot_bytes
    | Number
    | default = DefaultMaxHistorySnapshotBytes,
  missing
    | HistoryFallbackContract
    | default = 'structural,
  stale
    | HistoryFallbackContract
    | default = 'structural,
  incompatible
    | HistoryFallbackContract
    | default = 'structural,
  oversized
    | HistoryFallbackContract
    | default = 'structural,
} in

let ResourceBoundsContract = {
  max_classes | Number | default = DefaultMaxResourceClasses,
  max_class_name_bytes | Number | default = DefaultMaxResourceClassNameBytes,
  max_leases | Number | default = DefaultMaxResourceLeases,
  max_cpu_units | Number | default = DefaultMaxCpuUnits,
  max_resource_count | Number | default = DefaultMaxResourceCount,
  max_resource_bytes | Number | default = DefaultMaxResourceBytes,
  max_placement_candidates | Number | default = DefaultMaxPlacementCandidates,
} in

let LocalityPolicyContract = {
  max_observations | Number | default = DefaultMaxLocalityObservations,
  basis | [| 'receiver-verified |] | default = 'receiver-verified,
  stale | [| 'reject |] | default = 'reject,
  unverified | [| 'no-verified-content |] | default = 'no-verified-content,
} in

let SchedulingPolicyContract = {
  schema
    | String
    | default = "mantle-scheduling-policy-v1",
  policy_id
    | String
    | default = "mantle-lazy-priority-v1",
  preference_order
    | Array PreferenceFieldContract
    | default = ['known-graph, 'resource-fit, 'locality-transfer],
  aged_after_epochs
    | Number
    | default = DefaultAgedAfterEpochs,
  protected_after_epochs
    | Number
    | default = DefaultProtectedAfterEpochs,
} in

{
  SchedulingPolicy
    | doc "Runtime scheduling policy contract shared with the pure Rust priority kernel."
    = SchedulingPolicyContract,

  PreferenceField = PreferenceFieldContract,
  AgePolicy = AgePolicyContract,
  KnownGraphBounds = KnownGraphBoundsContract,
  HistoryPolicy = HistoryPolicyContract,
  ResourceBounds = ResourceBoundsContract,
  LocalityPolicy = LocalityPolicyContract,

  default_resource_bounds | ResourceBoundsContract = {},
  default_locality_policy | LocalityPolicyContract = {},

  default_policy
    | SchedulingPolicyContract
    = {},

  default_age_policy
    | AgePolicyContract
    = {},

  default_known_graph_bounds
    | KnownGraphBoundsContract
    = {},

  default_history_policy
    | HistoryPolicyContract
    = {},
}
