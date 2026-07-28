# Build Correctness Specification Delta

## ADDED Requirements

### Requirement: Dynamic-plan semantic values are nominal

r[build_correctness.dynamic_plan_nominal.values] Mantle MUST represent admitted unit IDs, source IDs, logical store paths, and output names with distinct private Rust types that enforce the existing scalar rules.

#### Scenario: Valid wire plan admits typed values

r[build_correctness.dynamic_plan_nominal.values.valid]
- GIVEN a `mantle-plan-v1` wire record contains bounded valid unit IDs, source IDs, logical store paths, and output names
- WHEN plan admission runs
- THEN Mantle MUST construct distinct admitted value types before graph validation.

#### Scenario: Invalid scalar fails before graph use

r[build_correctness.dynamic_plan_nominal.values.validation]
- GIVEN a plan contains an empty, oversized, control-bearing, malformed, or store-prefix-invalid semantic value
- WHEN plan admission runs
- THEN admission MUST fail with a deterministic scalar diagnostic
- AND graph validation MUST NOT receive the invalid value.

### Requirement: Dynamic-plan digest roles are nominal

r[build_correctness.dynamic_plan_nominal.digests] Mantle MUST represent canonical plan BLAKE3 identity and declared source NAR BLAKE3 identity with distinct Rust types.

#### Scenario: Plan and NAR digests do not compile interchangeably

r[build_correctness.dynamic_plan_nominal.compile_time]
- GIVEN plan and NAR digests use distinct marker-role instantiations
- WHEN source passes a NAR digest to an API that requires a plan digest
- THEN the source MUST fail compilation with a type mismatch.

### Requirement: Dynamic-plan wire and core models are separate

r[build_correctness.dynamic_plan_nominal.wire_boundary] Mantle MUST preserve the current `mantle-plan-v1` serialized shape in explicit wire DTOs and MUST convert admitted values through one pure wire-to-core boundary.

#### Scenario: Wire projection remains compatible

r[build_correctness.dynamic_plan_nominal.wire_boundary.compatible]
- GIVEN an admitted typed plan is projected for serialization
- WHEN Mantle emits `mantle-plan-v1`
- THEN field names, scalar spellings, enum tags, nullable fields, and collection shapes MUST match the accepted wire contract.

### Requirement: Dynamic-plan graph validation stays typed

r[build_correctness.dynamic_plan_nominal.graph] Mantle graph validation MUST retain typed unit, source, output, path, and digest values until diagnostics or wire projection require text.

#### Scenario: Unit and source IDs do not compile interchangeably

r[build_correctness.dynamic_plan_nominal.graph.ids]
- GIVEN `UnitId` and `SourceId` wrap the same scalar representation
- WHEN source uses `SourceId` for a unit edge
- THEN the source MUST fail compilation.

#### Scenario: Wrong-role placeholder fails

r[build_correctness.dynamic_plan_nominal.graph.validation]
- GIVEN a placeholder names a source where a unit output is required or names a unit where a source is required
- WHEN placeholder admission and graph validation run
- THEN Mantle MUST reject the placeholder with a deterministic role diagnostic.

### Requirement: Dynamic-plan canonical identity remains stable

r[build_correctness.dynamic_plan_nominal.compatibility] The nominal-type migration MUST preserve accepted canonical JSON bytes and plan BLAKE3 identities.

#### Scenario: Canonical plan bytes remain stable

r[build_correctness.dynamic_plan_nominal.compatibility.golden]
- GIVEN an accepted `mantle-plan-v1` fixture is processed before and after the migration
- WHEN canonical bytes and plan digests are compared
- THEN they MUST remain equal unless a separate versioned plan schema change approves the difference.

### Requirement: Octet checks the migrated dynamic-plan core

r[build_correctness.dynamic_plan_nominal.octet] After the policy becomes available, Mantle MUST declare its dynamic-plan domains to the reviewed Octet nominal-domain policy.

#### Scenario: Raw aliases do not return

r[build_correctness.dynamic_plan_nominal.octet.guard]
- GIVEN the dynamic-plan core has migrated to nominal types
- WHEN the Octet policy checks the configured scope
- THEN direct primitive aliases and raw declared domain values MUST fail the selected check.

### Requirement: Typed plan claims remain bounded

r[build_correctness.dynamic_plan_nominal.docs] Mantle documentation MUST state that typed plan values prove local category separation and scalar admission only.

#### Scenario: Boundary remains visible

r[build_correctness.dynamic_plan_nominal.final_checks]
- GIVEN a typed plan passes admission, graph checking, and canonicalization
- WHEN Mantle states the supported claim
- THEN it MUST NOT claim store presence, build success, sandbox enforcement, source trust, compiler correctness, or release eligibility.
