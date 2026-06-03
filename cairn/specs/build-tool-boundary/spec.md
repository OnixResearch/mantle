# Build Tool Boundary Specification

## Purpose

Defines Mantle's boundary as a frontend-neutral build tool, not an Onix/NixOS-style module layer.

## Requirements

### Requirement: Mantle build-tool boundary

r[build_tool_boundary.mantle_not_module_layer] Mantle MUST remain a frontend-neutral build tool and MUST NOT own Onix/NixOS-style module-layer semantics.

#### Scenario: external frontend hands Mantle build inputs

GIVEN an external frontend such as Onix has evaluated its own module layer and produced concrete derivations or build-plan inputs
WHEN it invokes Mantle
THEN Mantle MUST realize those build inputs through build/store APIs
AND Mantle MUST NOT require machine-role, tag-expansion, provider-policy, upstream-export, or settings-contract semantics to be present in Mantle core.

#### Scenario: raw module-layer input is not a Mantle build input

GIVEN a caller passes raw Onix inventory/module concepts directly to Mantle's build-tool boundary
WHEN Mantle validates the request
THEN Mantle MUST fail or route the request to an explicitly external module-layer tool
AND it MUST NOT silently interpret raw Onix roles, tags, providers, packages, artifacts, or upstream exports.

#### Scenario: implementation surface is guarded against module-layer coupling

GIVEN Mantle implementation or public handoff files change
WHEN boundary tests scan the CLI, stdlib, workspace metadata, implementation sources, docs, and examples
THEN they MUST reject reintroduced in-tree module-layer surfaces such as `mantle system`, `crunch-system`, `SystemModule`, Onix module repos, or NixOS module evaluators
AND they MUST keep accepted frontend handoff examples build-shaped.

### Requirement: Onix-owned module lowering

r[build_tool_boundary.onix_owns_module_lowering] Onix or an Onix-owned adapter MUST own module ABI, module implementation invocation, settings validation, upstream/provider topology, package/artifact selection, and lowering into Mantle build inputs.

#### Scenario: Onix evaluates modules before Mantle build

GIVEN an Onix service module with role settings, defaults, contracts, upstream dependencies, and provider exports
WHEN Onix uses Mantle as its backend
THEN Onix MUST evaluate those module semantics before calling Mantle
AND Mantle MUST receive only frontend-neutral build inputs or opaque data produced by that evaluation.

#### Scenario: Onix diagnostics remain in the module layer

GIVEN invalid Onix settings or missing Onix upstream/provider data
WHEN the Onix module layer evaluates the configuration
THEN diagnostics MUST be produced by the Onix-owned module layer
AND Mantle MUST NOT need Onix-specific diagnostic rules to report those module-layer failures.

### Requirement: Synthetic system scaffold is not an integration contract

r[build_tool_boundary.synthetic_system_eval_not_integration] Mantle MUST NOT treat the current synthetic `system eval` scaffold as the production Onix module integration path.

#### Scenario: fake fragments are not deployable evidence

GIVEN Mantle's current `system eval` path returns fragments that were not produced by real module implementations
WHEN deployable artifact work is planned
THEN those fragments MUST NOT be used as evidence that Onix modules run on Mantle
AND the work MUST depend on an Onix-owned module layer that lowers into Mantle build inputs.

#### Scenario: system scaffold is quarantined

GIVEN the `system eval` scaffold remains in the Mantle tree
WHEN operators or tests describe supported Onix integration
THEN the scaffold MUST be documented or guarded as experimental/demo-only
AND it MUST NOT expose a stable Onix/NixOS-style module ABI from Mantle core.

### Requirement: Mantle admits frontend artifact specs generically [r[build_tool_boundary.frontend_artifact_spec_admission]]

Mantle MUST provide a frontend-neutral artifact spec admission mechanism. A frontend MAY supply a content-addressed artifact spec and validator reference for a built output, and Mantle MUST validate the artifact manifest against that spec before reporting the artifact as spec-admitted. Mantle MUST NOT require frontend-specific module-layer semantics in core to perform this admission.

#### Scenario: Frontend spec validates an artifact [r[build_tool_boundary.frontend_artifact_spec_admission.scenario.valid]]

GIVEN a frontend build input declares a spec id, version, validator kind, validator ref, BLAKE3 spec hash, artifact kind, and artifact manifest
WHEN Mantle builds the output and runs spec admission
THEN Mantle MUST validate the artifact manifest against the declared spec
AND the result MUST identify the spec, validator, artifact ref, and validation outcome.

#### Scenario: Missing spec binding fails [r[build_tool_boundary.frontend_artifact_spec_admission.scenario.missing-spec]]

GIVEN an artifact manifest claims a frontend-owned artifact kind without a spec reference
WHEN Mantle evaluates spec admission
THEN Mantle MUST fail closed or mark the artifact not admitted
AND diagnostics MUST state that the frontend artifact kind lacks spec proof.

#### Scenario: Unsupported validator fails [r[build_tool_boundary.frontend_artifact_spec_admission.scenario.unsupported-validator]]

GIVEN a spec reference declares a validator kind Mantle does not support
WHEN Mantle evaluates spec admission
THEN Mantle MUST reject the admission attempt
AND diagnostics MUST name the unsupported validator kind.

### Requirement: Spec validation is attested in build reports [r[build_tool_boundary.spec_validation_attestation]]

Mantle build reports and artifact receipt material MUST record spec-validation attestation for spec-admitted frontend artifacts. The attestation MUST bind spec id, spec version, validator kind, validator reference or digest, spec hash, artifact ref or digest, validation result, and build provenance.

#### Scenario: Build report carries validation attestation [r[build_tool_boundary.spec_validation_attestation.scenario.report]]

GIVEN an artifact validates against a frontend-provided spec
WHEN Mantle emits a JSON build report
THEN the report MUST include spec-validation attestation fields
AND a caller MUST be able to identify the spec id, version, hash, validator kind, artifact ref, validation result, and provenance.

#### Scenario: Hash mismatch rejects attestation [r[build_tool_boundary.spec_validation_attestation.scenario.hash-mismatch]]

GIVEN a spec reference hash does not match the validator or spec material supplied to Mantle
WHEN validation would otherwise run
THEN Mantle MUST reject the artifact admission
AND it MUST NOT emit a successful validation attestation.

### Requirement: Frontend artifact kinds remain data, not Mantle built-ins [r[build_tool_boundary.frontend_agnostic_artifact_kinds]]

Mantle MUST treat frontend artifact `kind` strings as data governed by frontend specs. Mantle core MUST NOT hard-code behavior for Onix activation, NixOS system, service role, provider, tag, upstream export, or other frontend-specific artifact kinds.

#### Scenario: Onix-like kind is accepted only under spec [r[build_tool_boundary.frontend_agnostic_artifact_kinds.scenario.onix-kind-data]]

GIVEN an artifact manifest uses kind `mantle-onix-activation-closure`
WHEN Mantle processes the build result
THEN Mantle MUST treat the kind as frontend data
AND it MUST accept the artifact only if the declared frontend spec validates the manifest.

#### Scenario: Core semantic coupling is rejected [r[build_tool_boundary.frontend_agnostic_artifact_kinds.scenario.no-core-coupling]]

GIVEN Mantle core code, public examples, or stable APIs change
WHEN boundary checks inspect the change
THEN they MUST reject built-in interpretation of Onix roles, tags, providers, upstream exports, settings contracts, NixOS systems, or Onix activation policies
AND frontend-specific semantics MUST remain in frontend specs or adapters outside Mantle core.
