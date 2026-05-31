## Context

The existing worker already supports build-time graph growth through lazy goals and a compatibility detector that parses `.drv` ATerm outputs. That proves the scheduling seam is viable, but it keeps the dynamic story tied to Nix internals.

Mantle's native model treats dynamic derivations as declared build-plan data produced by a sandboxed build, validated by Rust, then scheduled by the existing worker. Nickel remains the declarative authoring language. Steel is intentionally excluded for this slice.

## Goals / Non-Goals

**Goals:**

- Define a native `mantle-plan-v1` JSON data ABI owned by Mantle.
- Keep the validator as a pure Rust functional core with deterministic output.
- Require producers to declare dynamic-plan outputs before build dispatch.
- Register accepted units through the existing lazy worker.
- Record deterministic BLAKE3 provenance for accepted and rejected plans.

**Non-Goals:**

- Add Steel.
- Implement Nix IFD or evaluator suspension.
- Remove the existing `.drv` compatibility path.
- Build a package-manager-specific resolver.
- Trust plans produced outside the current sandbox without a later trust design.

## Decisions

### 1. Dynamic plans are canonical JSON data, not evaluator re-entry

**Choice:** A producer emits a UTF-8 JSON artifact whose top-level `schema` field is exactly `mantle-plan-v1`. The worker decodes, validates, canonicalizes, and registers units. Nickel evaluation is not resumed during this flow.

**Wire encoding:** The artifact is a JSON object with snake_case field names. Enums use adjacent `kind` tagging, for example `{ "kind": "store_path", "path": "/mantle/store/..." }`. Unknown fields are rejected at every object level.

**Identifier grammar:**

- `UnitId` and `SourceId` must match `^[a-z][a-z0-9_.-]*$` and fit `MAX_DYNAMIC_PLAN_ID_BYTES = 128`.
- Output names must match `^[a-z][a-z0-9_+-]*$` and fit `MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES = 64`.
- `BLAKE3Hex` must be exactly `BLAKE3_HEX_BYTES = 64` lowercase hex characters.
- `StorePathString` must start with the active logical store prefix, contain a valid first store-path component, and any suffix after that component must be a normalized relative path with no empty, `.` or `..` components.

**Required fields:**

```text
DynamicPlanV1 {
  schema: "mantle-plan-v1",
  producer: PlanProducer,
  sources: Vec<DeclaredSourceInput>,
  units: Vec<DynamicUnit>,
  roots: Vec<UnitId>,
  provenance: BTreeMap<String, String>,
}

PlanProducer {
  logical_name: String,
  goal_hint: Option<String>,
}

DynamicUnit {
  id: UnitId,
  derivation: DynamicDerivation,
  requested_outputs: Vec<String>,
  policy: DynamicUnitPolicy,
}

DynamicUnitPolicy {
  sandbox: "inherit",
  substitutions: "inherit",
  store_prefix: "inherit",
  host_paths: "none",
}

DynamicDerivation {
  name: String,
  builder: StorePathString,
  system: String,
  args: Vec<String>,
  outputs: Vec<String>,
  env: BTreeMap<String, String>,
  inputs: Vec<DynamicInput>,
  fixed_output: Option<FixedOutputSpec>,
  addressing_mode: "content-addressed" | "input-addressed",
  sandbox: "native",
  dynamic_plan_outputs: Vec<String>,
}

DeclaredSourceInput {
  id: SourceId,
  path: StorePathString,
  nar_blake3: Option<BLAKE3Hex>,
}

FixedOutputSpec {
  mode: "flat" | "recursive",
  algo: "sha256" | "sha512" | "sha1" | "md5" | "blake3",
  hash: String,
}

DynamicInput =
  StorePath { kind: "store_path", path: StorePathString }
  | Source { kind: "source", source: SourceId }
  | UnitOutput { kind: "unit_output", unit: UnitId, output: String }
```

**Named limits:**

- `MAX_DYNAMIC_PLAN_BYTES = 4 MiB`
- `MAX_DYNAMIC_PLAN_UNITS = 4096`
- `MAX_DYNAMIC_PLAN_DEPENDENCIES_PER_UNIT = 256`
- `MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT = 16`
- `MAX_DYNAMIC_PLAN_ENV_ENTRIES_PER_UNIT = 512`
- `MAX_DYNAMIC_PLAN_STRING_BYTES = 16384`
- `MAX_DYNAMIC_PLAN_NESTING_DEPTH = 16`
- `MAX_DYNAMIC_PLAN_ID_BYTES = 128`
- `MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES = 64`
- `BLAKE3_HEX_BYTES = 64`

**Canonicalization:** Decode JSON into owned Rust structs, reject unknown fields, normalize set-like collections, then serialize compact JSON from canonical structs. Canonical order is: sources by `id`, units by `id`, roots by `id`, outputs by name, `dynamic_plan_outputs` by name, environment/provenance by key, and inputs by `(kind, source/unit/path, output)`. Ordered command arguments remain in original order. The accepted plan digest is BLAKE3 over those canonical bytes.

**Rationale:** JSON is easy for Nickel and Rust tools to emit now. Canonical Rust-side bytes avoid formatting/key-order drift while keeping the scheduler independent of Nickel.

**Alternative:** Implement Nix-style import-from-derivation. Rejected because it requires evaluator suspension/resumption and creates a much larger trust boundary.

### 2. Producers declare exact dynamic-plan outputs with `dynamic_plan_outputs`

**Choice:** The derivation model gains `dynamic_plan_outputs: Vec<String>` in Rust and `dynamic_plan_outputs | Array String | default = []` in the Nickel derivation contract.

**Validation:** Every declared dynamic-plan output name must also appear in `outputs`. The worker scans only those output names. If a declared output is missing, not a regular file, larger than `MAX_DYNAMIC_PLAN_BYTES`, or invalid JSON/plan data, the worker records a structured rejection keyed by producer goal and output name and registers no units from that artifact. Outputs not named in `dynamic_plan_outputs` are ignored by the native dynamic-plan scanner and produce no native dynamic-plan report row.

**Rationale:** Declaration avoids scanning arbitrary outputs, prevents accidental scheduling, and gives the worker a fail-closed policy surface.

**Alternative:** Content-sniff every output. Rejected because false positives and hidden graph growth are unacceptable for a native ABI.

### 3. Dynamic units inherit policy; MVP permits no widening fields

**Choice:** MVP dynamic units inherit the producer's sandbox, substitute/trust policy, store prefix, and host-path boundary. Plan data may only narrow through unit-local build shape; it may not declare new trust roots or broader host access.

**Concrete rules:**

- `sandbox` must be `native`, matching the only currently shipped sandbox contract.
- `DynamicUnitPolicy` fields must equal the literal inheritance/no-host-path values shown in the ABI; any other value is rejected as policy widening.
- Dynamic plans carry no trusted-key, substitute, `trust_unsigned`, or remote-cache fields beyond the literal inheritance policy; unknown policy fields are rejected.
- Store paths in `builder` and `DynamicInput::StorePath` must use the active logical store prefix.
- `DynamicInput::Source` may reference only a top-level `DeclaredSourceInput` in the same accepted plan.
- `DeclaredSourceInput.path` must use the active logical store prefix, and `nar_blake3` must be absent or a valid lowercase BLAKE3 hex digest; the registration shell later rejects the source if local PathInfo/castore cannot satisfy the declared path/digest.
- `DynamicInput::UnitOutput` may reference only a unit in the same accepted plan and an output declared by that unit.
- Absolute paths outside the active logical store prefix are rejected in v1.
- A nested dynamic unit may declare its own `dynamic_plan_outputs`, but total graph growth is still bounded by worker goal limits and the named plan limits.

**Rationale:** The safest first version is inheritance plus rejection of widening syntax. Later policy posets can be added without weakening v1.

### 4. Native dynamic scheduling reuses lazy goals

**Choice:** Accepted units are converted into the same build-engine registry entry shape used by normal evaluated derivations, then scheduled through existing `Worker::want`/ready-queue behavior.

**Root and reachability rules:** `roots` must be non-empty, unique, and every root must reference an existing `DynamicUnit.id`. All units are validated and inserted into the registry. The worker calls `want()` only for the root units. Root dependency closure is therefore built in the same run; non-root units that are not reachable from roots may remain registered but not built. Unit-output dependency cycles anywhere in the plan are rejected by the validator before registration.

**Rationale:** The scheduler already supports mid-run graph growth. Reusing it avoids a second scheduler path. Registering the full accepted package set while wanting only roots lets future package-set generators emit more units than the immediate root closure without forcing every unit to build.

**Alternative:** Create a separate dynamic queue outside `GoalRegistry`. Rejected because it would duplicate dependency wiring and failure propagation.

**Implementation:** Add a native-plan registration function beside the existing `.drv` compatibility detector. Keep compatibility labels distinct in reports.

### 5. Provenance records native and compatibility modes separately

**Choice:** Each scanned declared output produces one deterministic report row:

```text
DynamicPlanReportRow {
  mode: "native-plan-v1" | "compat-nix-drv",
  producer_goal_id: String,
  output_name: String,
  plan_artifact_path: Option<StorePathString>,
  raw_artifact_digest: Option<BLAKE3Hex>,
  canonical_plan_digest: Option<BLAKE3Hex>,
  accepted_unit_ids: Vec<UnitId>,
  rejection_reason: Option<DynamicPlanErrorKind>,
  scheduler_action: "registered" | "rejected" | "ignored",
}
```

Rows are sorted by `(producer_goal_id, output_name, mode)`. Accepted unit IDs are sorted. Compatibility `.drv` discovery keeps its existing behavior but reports `mode = "compat-nix-drv"`.

Artifact-path rules: accepted plans and rejected declared outputs that produced a build output record `plan_artifact_path` as that output's logical store path. Missing declared outputs record no artifact path because no output artifact exists.

Digest rules: accepted plans record both `raw_artifact_digest` and `canonical_plan_digest`; decoded-but-invalid regular files under `MAX_DYNAMIC_PLAN_BYTES` record only `raw_artifact_digest`; missing outputs, non-regular outputs, and oversized outputs record no digest because the worker does not have bounded plan bytes to hash.

## Risks / Trade-offs

**Plan ABI churn** → Mitigate with explicit `schema = "mantle-plan-v1"`, typed rejection for unknown versions, and tests that freeze minimal valid/invalid fixtures.

**Unbounded graph growth** → Mitigate with named constants for bytes, units, dependencies, outputs, env entries, string bytes, nesting depth, and existing worker goal limits.

**Policy widening** → Mitigate by rejecting widening syntax in v1 and inheriting producer sandbox/trust/store-prefix policy.

**Nix compatibility confusion** → Mitigate by labeling `.drv` discovery as compatibility mode and making native plan output declarations the only core API.

## Validation Plan

- Pure positive tests for valid `mantle-plan-v1` decode, canonical digest, and unit extraction.
- Pure negative tests for unknown schema, unknown fields, duplicate IDs, empty required fields, undeclared dependencies, over-limit collections, invalid output names, invalid store-prefix references, absolute host paths, nesting-depth violations, and policy-widening fields.
- Worker integration test where one producer emits a valid plan and the discovered unit builds in the same run.
- Worker integration tests where missing, non-regular, oversized, invalid, undeclared, and policy-widening outputs fail closed or are ignored as specified.
- Report/provenance test proving producer goal ID, output name, plan artifact path when present, scheduler action, native-vs-compat labels, sorted unit IDs, and deterministic digest recording.
