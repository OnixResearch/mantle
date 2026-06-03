## ADDED Requirements

### Requirement: Mantle admits frontend artifact specs generically [r[build_tool_boundary.frontend_artifact_spec_admission]]

Mantle MUST provide a frontend-neutral artifact spec admission mechanism. A frontend MAY supply a content-addressed artifact spec and validator reference for a built output, and Mantle MUST validate the artifact manifest against that spec before reporting the artifact as spec-admitted. Mantle MUST NOT require frontend-specific module-layer semantics in core to perform this admission.

#### Scenario: Frontend spec validates an artifact [r[build_tool_boundary.frontend_artifact_spec_admission.scenario.valid]]

- GIVEN a frontend build input declares a spec id, version, validator kind, validator ref, BLAKE3 spec hash, artifact kind, and artifact manifest
- WHEN Mantle builds the output and runs spec admission
- THEN Mantle MUST validate the artifact manifest against the declared spec
- AND the result MUST identify the spec, validator, artifact ref, and validation outcome.

#### Scenario: Missing spec binding fails [r[build_tool_boundary.frontend_artifact_spec_admission.scenario.missing-spec]]

- GIVEN an artifact manifest claims a frontend-owned artifact kind without a spec reference
- WHEN Mantle evaluates spec admission
- THEN Mantle MUST fail closed or mark the artifact not admitted
- AND diagnostics MUST state that the frontend artifact kind lacks spec proof.

#### Scenario: Unsupported validator fails [r[build_tool_boundary.frontend_artifact_spec_admission.scenario.unsupported-validator]]

- GIVEN a spec reference declares a validator kind Mantle does not support
- WHEN Mantle evaluates spec admission
- THEN Mantle MUST reject the admission attempt
- AND diagnostics MUST name the unsupported validator kind.

### Requirement: Spec validation is attested in build reports [r[build_tool_boundary.spec_validation_attestation]]

Mantle build reports and artifact receipt material MUST record spec-validation attestation for spec-admitted frontend artifacts. The attestation MUST bind spec id, spec version, validator kind, validator reference or digest, spec hash, artifact ref or digest, validation result, and build provenance.

#### Scenario: Build report carries validation attestation [r[build_tool_boundary.spec_validation_attestation.scenario.report]]

- GIVEN an artifact validates against a frontend-provided spec
- WHEN Mantle emits a JSON build report
- THEN the report MUST include spec-validation attestation fields
- AND a caller MUST be able to identify the spec id, version, hash, validator kind, artifact ref, validation result, and provenance.

#### Scenario: Hash mismatch rejects attestation [r[build_tool_boundary.spec_validation_attestation.scenario.hash-mismatch]]

- GIVEN a spec reference hash does not match the validator or spec material supplied to Mantle
- WHEN validation would otherwise run
- THEN Mantle MUST reject the artifact admission
- AND it MUST NOT emit a successful validation attestation.

### Requirement: Frontend artifact kinds remain data, not Mantle built-ins [r[build_tool_boundary.frontend_agnostic_artifact_kinds]]

Mantle MUST treat frontend artifact `kind` strings as data governed by frontend specs. Mantle core MUST NOT hard-code behavior for Onix activation, NixOS system, service role, provider, tag, upstream export, or other frontend-specific artifact kinds.

#### Scenario: Onix-like kind is accepted only under spec [r[build_tool_boundary.frontend_agnostic_artifact_kinds.scenario.onix-kind-data]]

- GIVEN an artifact manifest uses kind `mantle-onix-activation-closure`
- WHEN Mantle processes the build result
- THEN Mantle MUST treat the kind as frontend data
- AND it MUST accept the artifact only if the declared frontend spec validates the manifest.

#### Scenario: Core semantic coupling is rejected [r[build_tool_boundary.frontend_agnostic_artifact_kinds.scenario.no-core-coupling]]

- GIVEN Mantle core code, public examples, or stable APIs change
- WHEN boundary checks inspect the change
- THEN they MUST reject built-in interpretation of Onix roles, tags, providers, upstream exports, settings contracts, NixOS systems, or Onix activation policies
- AND frontend-specific semantics MUST remain in frontend specs or adapters outside Mantle core.
