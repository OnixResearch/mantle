# Nickel contracts for cairn policy profiles.

let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in

let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in

let SupportedPolicyVersion = std.contract.from_predicate (fun value =>
  std.is_number value && value == 1
) in

let Blake3Algorithm = std.contract.from_predicate (fun value =>
  std.is_string value && value == "BLAKE3"
) in

let Canonicalization = std.contract.from_predicate (fun value =>
  std.is_string value && value == "serde_json_compact_sorted_keys"
) in

let Blake3Hex = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-fA-F]{64}$" value
) in

let Blake3HexContract = Blake3Hex in

let Stage = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "proposal" || value == "design" || value == "tasks" || value == "implementation" || value == "archive")
) in

let Strictness = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "strict" || value == "warn" || value == "off")
) in

let MarkerToken = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^\\[(parallel|serial|depends:[^\\]]+)\\]$" value
) in

let MarkerKind = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "literal" || value == "depends-prefix")
) in

let Severity = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "low" || value == "medium" || value == "high" || value == "critical")
) in

let FindingClass = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "omission" || value == "systematic" || value == "incoherent" || value == "preference" || value == "env" || value == "pattern")
) in

let FindingScope = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "proposal" || value == "spec" || value == "design" || value == "tasks" || value == "review" || value == "tooling")
) in

let FindingRoute = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "auto-fix" || value == "prompt" || value == "deterministic-check" || value == "spec-rule" || value == "human" || value == "guidance")
) in

let Promoted = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "yes" || value == "no")
) in

let SteelProfileMode = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "disabled" || value == "comparison" || value == "opt_in" || value == "selected_default" || value == "broad_default")
) in

let SafeRelativePath = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0 && std.string.is_match "^[A-Za-z0-9][A-Za-z0-9._/-]*$" value
) in

let TraceabilityRequirementSourceKind = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "accepted_specs" || value == "requirements_doc")
) in

let TraceabilityEvidenceSourceKind = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "rust_source" || value == "nickel_source" || value == "tool_source")
) in

let TraceabilityMarkerVerb = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "impl" || value == "verify" || value == "depends" || value == "related")
) in

let TraceabilityExtension = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0 && std.string.is_match "^[A-Za-z0-9_-]+$" value
) in

let StackProvenanceGateMode = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "disabled" || value == "optional" || value == "required")
) in

let RuntimeCaptureOrigin = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "boundary_captured" || value == "ingested_telemetry" || value == "wrapper_asserted" || value == "unknown")
) in

let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in

{
  Artifact = {
    id | NonEmptyString,
    generates | NonEmptyString,
    requires | Array NonEmptyString | default = [],
  },

  ArtifactSchema = {
    id | NonEmptyString,
    description | NonEmptyString,
    artifacts | Array Artifact,
  },

  Marker = {
    id | NonEmptyString,
    token | MarkerToken,
    kind | MarkerKind,
  },

  TaskMarkerPolicy = {
    enabled | Bool,
    stages | NonEmptyArray | Array Stage,
    strictness | Strictness,
    ignore_archives | Bool,
    markers | NonEmptyArray | Array Marker,
  },

  FindingPolicy = {
    severity | Severity,
    class | FindingClass,
    scope | FindingScope,
    route | FindingRoute,
    promoted | Promoted,
  },

  GatePolicy = {
    stages | NonEmptyArray | Array Stage,
    ignore_archives | Bool,
    finding | FindingPolicy,
  },

  ReportContract = {
    verdicts | NonEmptyArray | Array NonEmptyString,
    sections | NonEmptyArray | Array NonEmptyString,
    tagged_findings_required_fields | NonEmptyArray | Array NonEmptyString,
  },

  RegistryLabelPolicy = {
    channels | NonEmptyArray | Array NonEmptyString,
    platforms | NonEmptyArray | Array NonEmptyString,
    compatibility_classes | NonEmptyArray | Array NonEmptyString,
    release_format | NonEmptyString,
  },

  ChangeMetadataPolicy = {
    allowed_groups | NonEmptyArray | Array NonEmptyString,
    group_prefixes | Array NonEmptyString | default = [],
    statuses | NonEmptyArray | Array NonEmptyString,
  },

  SteelOrchestrationProfile = {
    id | NonEmptyString,
    mode | SteelProfileMode,
    allowed_host_functions | Array NonEmptyString | default = [],
    deterministic_budgets | NonEmptyArray | Array NonEmptyString,
    redaction_classes | NonEmptyArray | Array NonEmptyString,
    fallback_modes | NonEmptyArray | Array NonEmptyString,
    receipt_schema_version | NonEmptyString,
  },

  SteelOrchestrationPolicy = {
    enabled | Bool,
    profiles | Array SteelOrchestrationProfile | default = [],
  },

  TraceabilityRequirementSource = {
    root | SafeRelativePath,
    source_kind | TraceabilityRequirementSourceKind,
    extensions | NonEmptyArray | Array TraceabilityExtension,
  },

  TraceabilityEvidenceSource = {
    root | SafeRelativePath,
    source_kind | TraceabilityEvidenceSourceKind,
    extensions | NonEmptyArray | Array TraceabilityExtension,
  },

  TraceabilityProfile = {
    id | NonEmptyString,
    requirement_sources | NonEmptyArray | Array TraceabilityRequirementSource,
    evidence_sources | NonEmptyArray | Array TraceabilityEvidenceSource,
    marker_verbs | NonEmptyArray | Array TraceabilityMarkerVerb,
  },

  TraceabilityPolicy = {
    default_profile | NonEmptyString,
    profiles | NonEmptyArray | Array TraceabilityProfile,
  },

  StackProvenancePolicy = {
    mode | StackProvenanceGateMode,
    allowed_claim_scopes | NonEmptyArray | Array NonEmptyString,
    accepted_valence_policy_hashes | Array Blake3HexContract | default = [],
    required_boundary | NonEmptyString,
  },

  FunctionAddressPolicy = {
    mode | StackProvenanceGateMode,
    allowed_claim_scopes | NonEmptyArray | Array NonEmptyString,
    accepted_valence_policy_hashes | Array Blake3HexContract | default = [],
    required_roles | NonEmptyArray | Array NonEmptyString,
    required_schemas | NonEmptyArray | Array NonEmptyString,
    required_boundary | NonEmptyString,
  },

  RuntimeEvidencePolicy = {
    allowed_capture_origins | NonEmptyArray | Array RuntimeCaptureOrigin,
    allowed_hash_algorithms | NonEmptyArray | Array Blake3Algorithm,
    max_summary_bytes | PositiveNumber,
    forbidden_summary_fragments | Array NonEmptyString | default = [],
    aggregation_profiles | Array NonEmptyString | default = [],
  },

  ProbeCommandProfile = {
    id | NonEmptyString,
    commands | NonEmptyArray | Array NonEmptyString,
    capability_classes | Array NonEmptyString | default = [],
  },

  ProbePolicy = {
    enabled | Bool,
    inline_gate_execution_enabled | Bool,
    receipt_schema_version | NonEmptyString,
    coverage_classes | NonEmptyArray | Array NonEmptyString,
    command_profiles | NonEmptyArray | Array ProbeCommandProfile,
    redaction_classes | NonEmptyArray | Array NonEmptyString,
  },

  ReceiptSchema = {
    command | NonEmptyString,
    top_level_keys | NonEmptyArray | Array NonEmptyString,
    action_keys | Array NonEmptyString | default = [],
    issue_keys | Array NonEmptyString | default = [],
    case_keys | Array NonEmptyString | default = [],
    mismatch_keys | Array NonEmptyString | default = [],
    artifact_keys | Array NonEmptyString | default = [],
    replay_evidence_keys | Array NonEmptyString | default = [],
    hash_fields | Array NonEmptyString | default = [],
  },

  ReceiptContract = {
    family | NonEmptyString,
    schema | NonEmptyString,
    command | NonEmptyString,
    ownership | NonEmptyString,
    compatibility_class | NonEmptyString,
    encoding | NonEmptyString,
    hash_field | NonEmptyString,
    hash_material | NonEmptyString,
    non_claims | NonEmptyArray | Array NonEmptyString,
  },

  ReplayCase = {
    id | NonEmptyString,
    command_kind | NonEmptyString,
    args | NonEmptyArray | Array NonEmptyString,
    fixture_root | NonEmptyString,
    expected_mutability | NonEmptyString,
    comparison_modes | NonEmptyArray | Array NonEmptyString,
    display_only_paths | Array NonEmptyString | default = [],
    hash_fields | Array NonEmptyString | default = [],
    normalization_rules | NonEmptyArray | Array NonEmptyString,
  },

  ReplayGroup = {
    id | NonEmptyString,
    description | NonEmptyString,
    cases | NonEmptyArray | Array NonEmptyString,
  },

  EntropyInventoryEntry = {
    id | NonEmptyString,
    class | NonEmptyString,
    source | NonEmptyString,
    handling | NonEmptyString,
    receipt_field | NonEmptyString,
    description | NonEmptyString,
  },

  EnvironmentAttestationFact = {
    id | NonEmptyString,
    class | NonEmptyString,
    source | NonEmptyString,
    handling | NonEmptyString,
    receipt_field | NonEmptyString,
    description | NonEmptyString,
  },

  DeterminismReplayPolicy = {
    cases | NonEmptyArray | Array ReplayCase,
    groups | NonEmptyArray | Array ReplayGroup,
    entropy_inventory | NonEmptyArray | Array EntropyInventoryEntry,
    environment_attestation | NonEmptyArray | Array EnvironmentAttestationFact,
  },

  DeterminismSurface = {
    id | NonEmptyString,
    command | NonEmptyString,
    surface_kind | NonEmptyString,
    receipt_command | NonEmptyString,
    required_replay_cases | NonEmptyArray | Array NonEmptyString,
    required_replay_groups | NonEmptyArray | Array NonEmptyString,
    determinism_claim | NonEmptyString,
  },

  ExemptionMarkerPolicy = {
    required_fields | NonEmptyArray | Array NonEmptyString,
    date_fields | NonEmptyArray | Array NonEmptyString,
    optional_fields | Array NonEmptyString | default = [],
  },

  SyncArchivePolicy = {
    require_all_tasks_done | Bool,
    archive_date_prefix | Bool,
    default_archive_dir | NonEmptyString,
  },

  MutationGate = {
    require_passing_gate | Bool,
  },

  MigrationPolicy = {
    require_manifest | Bool,
  },

  MutationPolicy = {
    sync | MutationGate,
    archive | MutationGate,
    migrate | MigrationPolicy,
  },

  ReceiptHashPolicy = {
    required | Bool,
    algorithm | Blake3Algorithm,
    canonicalization | Canonicalization,
  },

  PolicySchemaCompatibility = {
    schema_version | NonEmptyString,
    producer | NonEmptyString,
    field_set_digest | Blake3HexContract,
    refresh_command | NonEmptyString,
  },

  Exemption = {
    scope | NonEmptyString,
    owner | NonEmptyString,
    reason | NonEmptyString,
    expires | NonEmptyString,
  },

  PreservesReceiptSourceMap = { _ : NonEmptyString },

  StructuredLifecycle = {
    preserves_receipt_source_map | PreservesReceiptSourceMap | default = {},
  },

  Policy = {
    name | NonEmptyString,
    version | SupportedPolicyVersion,
    policy_schema_compatibility | PolicySchemaCompatibility,
    schemas | NonEmptyArray | Array ArtifactSchema,
    lifecycle_stages | NonEmptyArray | Array Stage,
    task_marker_policy | TaskMarkerPolicy,
    gate_policy | GatePolicy,
    finding_defaults | FindingPolicy,
    report_contract | ReportContract,
    registry_label_policy | RegistryLabelPolicy,
    change_metadata_policy | ChangeMetadataPolicy,
    steel_orchestration_policy | SteelOrchestrationPolicy,
    traceability_policy | TraceabilityPolicy,
    stack_provenance_gate | StackProvenancePolicy,
    function_address_gate | FunctionAddressPolicy,
    runtime_evidence_policy | RuntimeEvidencePolicy,
    structured_lifecycle | StructuredLifecycle | default = {},
    probe_policy | ProbePolicy,
    receipt_schemas | NonEmptyArray | Array ReceiptSchema,
    receipt_contracts | Array ReceiptContract | default = [],
    determinism_replay_policy | DeterminismReplayPolicy,
    determinism_surface_inventory | NonEmptyArray | Array DeterminismSurface,
    sync_archive_policy | SyncArchivePolicy,
    mutation_policy | MutationPolicy,
    receipt_hash_policy | ReceiptHashPolicy,
    exemption_marker_policy | ExemptionMarkerPolicy,
    exemptions | Array Exemption,
  },

  Blake3Hex = Blake3Hex,
}
