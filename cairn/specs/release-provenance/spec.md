# Release Provenance Specification

## Purpose

Defines the `release-provenance` capability.

## Requirements

### Requirement: Valence stack provenance requirement policy
r[mantle.release_provenance.valence_required_policy] Mantle release profiles SHOULD be able to declare Valence stack-provenance sidecar evidence as optional or required, and required mode MUST fail closed when the sidecar or its Valence verification receipt is missing or invalid.

#### Scenario: Optional absent sidecar is recorded as absent
r[mantle.release_provenance.valence_required_policy.optional_absent]
- GIVEN a release profile where stack provenance is optional
- WHEN Mantle verifies a release bundle without a stack-provenance sidecar
- THEN verification MUST record an absent or skipped stack-provenance disposition without claiming the bundle carries stack provenance.

#### Scenario: Required sidecar passes with Valence receipt
r[mantle.release_provenance.valence_required_policy.required_valid]
- GIVEN a release profile where stack provenance is required and the bundle carries a matching sidecar plus Valence verification receipt
- WHEN Mantle verifies the release bundle
- THEN verification MUST accept the bundle-local stack-provenance evidence wiring.

#### Scenario: Required sidecar missing fails closed
r[mantle.release_provenance.valence_required_policy.required_missing]
- GIVEN a release profile where stack provenance is required
- WHEN the bundle has no stack-provenance sidecar or Valence verification receipt
- THEN verification MUST fail with deterministic diagnostics naming the missing evidence.

### Requirement: Valence verification receipt binding
r[mantle.release_provenance.valence_receipt_binding] Mantle release evidence MUST bind the stack-provenance sidecar artifact, Valence verification receipt artifact, BLAKE3 hashes, external evidence role, schema, claim scope, release binary identity, and required non-claims.

#### Scenario: Sidecar digest is verified
r[mantle.release_provenance.valence_receipt_binding.sidecar_digest]
- GIVEN a release bundle declares a stack-provenance sidecar artifact
- WHEN Mantle verifies the bundle
- THEN the sidecar bytes MUST match the BLAKE3 digest recorded in release evidence metadata.

#### Scenario: Valence receipt digest is verified
r[mantle.release_provenance.valence_receipt_binding.valence_receipt]
- GIVEN a release bundle declares a Valence stack-provenance verification receipt
- WHEN Mantle verifies the bundle
- THEN the receipt bytes MUST match the BLAKE3 digest recorded in release evidence metadata.

#### Scenario: Binary identity is linked
r[mantle.release_provenance.valence_receipt_binding.binary_identity]
- GIVEN stack-provenance evidence is present in a release bundle
- WHEN Mantle verifies the bundle
- THEN the release binary identity in bundle metadata MUST match the binary identity named by the sidecar or Valence receipt metadata available to Mantle.

#### Scenario: Stale evidence fails closed
r[mantle.release_provenance.valence_receipt_binding.stale]
- GIVEN the sidecar bytes, Valence receipt bytes, role, schema, claim scope, or binary identity do not match declared metadata
- WHEN Mantle verifies required stack-provenance evidence
- THEN verification MUST fail with deterministic diagnostics naming the stale or mismatched field.

### Requirement: Mantle opaque stack-provenance boundary
r[mantle.release_provenance.opaque_boundary] Mantle MUST treat stack provenance sidecars as opaque Valence-validated external evidence and MUST NOT claim to verify Octet, Trellis, Valence, or Cairn stack semantics itself.

#### Scenario: Opaque boundary is visible
r[mantle.release_provenance.opaque_boundary.visible]
- GIVEN release verification output or operator documentation describes stack-provenance sidecar handling
- WHEN the supported claim is stated
- THEN it MUST say Mantle validates bundle-local path, digest, role, schema, claim scope, binary identity, and non-claims while Valence owns stack semantics.

#### Scenario: Overclaiming boundary fails closed
r[mantle.release_provenance.opaque_boundary.overclaim]
- GIVEN release metadata claims Mantle semantically verified Octet, Trellis, Valence, or Cairn provenance semantics
- WHEN Mantle verifies the release bundle
- THEN verification MUST fail or downgrade the claim with a deterministic overclaim diagnostic.

### Requirement: Release provenance fixture matrix
r[mantle.release_provenance.fixture_matrix] Mantle MUST include positive and negative fixtures for optional and required Valence stack-provenance release evidence.

#### Scenario: Positive fixtures cover optional and required modes
r[mantle.release_provenance.fixture_matrix.positive]
- GIVEN optional-absent, optional-present, and required-present release evidence fixtures
- WHEN the fixture suite runs
- THEN optional absent MUST record an absent disposition, optional present MUST verify bundle-local metadata, and required present MUST pass with matching Valence receipt evidence.

#### Scenario: Negative fixtures cover required-mode failures
r[mantle.release_provenance.fixture_matrix.negative]
- GIVEN fixtures with missing sidecar, wrong role, wrong schema, wrong claim scope, stale sidecar digest, missing Valence receipt, stale Valence receipt digest, missing binary identity, or weakened non-claims
- WHEN the fixture suite runs in required mode
- THEN each fixture MUST fail closed with deterministic diagnostics.

### Requirement: Stack release profile requires provenance
r[mantle.release_provenance.stack_profile.required] Mantle MUST support a release profile for Onix stack artifacts that requires Valence stack-provenance sidecar and graph-report evidence while preserving optional stack provenance for generic releases.

#### Scenario: Profile-present evidence passes
r[mantle.release_provenance.stack_profile.positive]
- GIVEN a stack release profile requires stack provenance
- AND the bundle includes a Valence sidecar, Valence graph report, matching BLAKE3 digests, expected roles, expected schemas, supported claim scope, release-binary identity, and required non-claims
- WHEN `mantle release verify` evaluates the profile
- THEN verification MUST pass the stack-provenance contribution.

#### Scenario: Required profile fails absent evidence
r[mantle.release_provenance.stack_profile.required_absent]
- GIVEN a stack release profile requires stack provenance
- AND the bundle omits the sidecar or Valence graph report
- WHEN release verification runs
- THEN verification MUST fail closed with deterministic diagnostics naming the missing evidence.

### Requirement: Shared stack-provenance constants
r[mantle.release_provenance.stack_profile.constants] Mantle MUST derive or validate stack-provenance role, schema, claim-scope, and required non-claim constants from a reviewed source so CLI text, manifest validation, and documentation do not drift.

#### Scenario: Constants match profile contract
r[mantle.release_provenance.stack_profile.constants.match]
- GIVEN release verification uses stack-provenance role, schema, claim-scope, and non-claim constants
- WHEN the constants check runs
- THEN each constant MUST match the reviewed stack release profile contract.

#### Scenario: Constant drift fails
r[mantle.release_provenance.stack_profile.constants.drift]
- GIVEN a Rust constant, CLI example, or documentation string diverges from the reviewed profile contract
- WHEN the constants check runs
- THEN validation MUST fail with a drift diagnostic.

### Requirement: Required profile negative fixture matrix
r[mantle.release_provenance.stack_profile.negative] Mantle MUST include fail-closed fixtures for required stack-provenance profile failures.

#### Scenario: Invalid evidence is rejected
r[mantle.release_provenance.stack_profile.negative.invalid]
- GIVEN required stack-provenance evidence has a stale digest, wrong role, wrong schema, wrong binary link, unsupported claim scope, missing Valence receipt, or weakened non-claims
- WHEN release verification evaluates the stack profile
- THEN verification MUST fail closed with diagnostics identifying the invalid evidence class.

### Requirement: Profile documentation
r[mantle.release_provenance.stack_profile.docs] Mantle operator documentation MUST distinguish generic optional stack provenance from required Onix stack release profiles.

#### Scenario: Generic release remains optional
r[mantle.release_provenance.stack_profile.docs.generic]
- GIVEN an operator reads generic Mantle release docs
- WHEN stack provenance is described
- THEN the docs MUST state that generic releases default to optional stack provenance.

#### Scenario: Stack profile boundary is visible
r[mantle.release_provenance.stack_profile.docs.boundary]
- GIVEN an operator reads stack release profile docs
- WHEN required stack provenance is described
- THEN the docs MUST state that Mantle validates bundle-local evidence linkage only and Valence owns stack semantics.

### Requirement: Validation evidence
r[mantle.release_provenance.stack_profile.validation] The change MUST include focused release-evidence tests, profile/constant checks, and Cairn validation evidence before archive.

#### Scenario: Validation covers pass and fail cases
r[mantle.release_provenance.stack_profile.validation.fixtures]
- GIVEN positive and negative stack profile fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass, invalid fixtures MUST fail closed, and saved evidence MUST bind profile and input identities.

### Requirement: Preserves carrier contract
r[mantle.release_provenance.preserves_carriers.contract] Mantle MUST validate Preserves release evidence carrier rows with explicit role, schema, payload digest, canonical digest, adapter identity when present, and bounded non-claims.

#### Scenario: Opaque and adapter-backed carriers pass
r[mantle.release_provenance.preserves_carriers.fixtures.positive]
- GIVEN an opaque Preserves carrier row and an adapter-backed carrier row with matching digest-shaped identities and required non-claims
- WHEN carrier validation runs
- THEN both rows MUST pass without turning Preserves payload semantics into Mantle-owned proof claims.

#### Scenario: Invalid carriers fail closed
r[mantle.release_provenance.preserves_carriers.fixtures.negative]
- GIVEN a Preserves carrier row has stale digest, wrong schema, wrong role, missing non-claims, or semantic overclaims
- WHEN carrier validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Preserves carrier validation boundary
r[mantle.release_provenance.preserves_carriers.validation] Mantle MUST integrate Preserves carrier validation as release external evidence checking over loaded rows while keeping file I/O, digesting, and payload acquisition in the shell.

#### Scenario: Carrier opacity remains visible
r[mantle.release_provenance.preserves_carriers.docs]
- GIVEN Preserves carrier validation succeeds
- WHEN Mantle reports or documents the evidence
- THEN the claim MUST be limited to supported carrier identity and digest evidence.
- AND the report MUST NOT claim release correctness, artifact correctness, deployment safety, full reproducibility, or semantic correctness from the carrier alone.

### Requirement: Preserves carrier final validation
r[mantle.release_provenance.preserves_carriers.final_validation] The Preserves carrier change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Focused carrier suite covers boundaries
- GIVEN valid and invalid Preserves carrier fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.

### Requirement: Cairn release evidence handoff contract
r[mantle.release_provenance.cairn_evidence_handoff.contract] Mantle MUST validate Cairn release evidence handoff rows with explicit artifact id, role, schema, artifact digest, Cairn policy digest, release-readiness id, coverage ids, and non-claims.

#### Scenario: Complete Cairn handoff passes
r[mantle.release_provenance.cairn_evidence_handoff.fixtures.positive]
- GIVEN Cairn release-readiness and archive-index handoff rows with supported roles, schemas, BLAKE3-shaped digests, coverage ids, and required non-claims
- WHEN handoff validation runs
- THEN validation MUST pass and preserve Cairn ownership of lifecycle readiness.

#### Scenario: Invalid Cairn handoff fails closed
r[mantle.release_provenance.cairn_evidence_handoff.fixtures.negative]
- GIVEN a handoff row has missing artifact id, stale digest, wrong role, wrong schema, or weakened non-claims
- WHEN handoff validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Cairn handoff validation boundary
r[mantle.release_provenance.cairn_evidence_handoff.validation] Mantle MUST integrate Cairn handoff validation as release external evidence checking over loaded rows while file reads, digest measurement, and Cairn export production remain outside the pure core.

#### Scenario: Mantle and Cairn ownership stays explicit
r[mantle.release_provenance.cairn_evidence_handoff.docs]
- GIVEN Cairn handoff validation succeeds
- WHEN Mantle reports or documents the evidence
- THEN the claim MUST be limited to bundle-local handoff identity and digest linkage.
- AND Mantle MUST NOT claim release correctness, build correctness, source correctness, artifact correctness, or deployment safety from Cairn handoff evidence alone.

### Requirement: Cairn handoff final validation
r[mantle.release_provenance.cairn_evidence_handoff.final_validation] The Cairn handoff change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Focused handoff suite covers boundaries
- GIVEN valid and invalid Cairn handoff fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.

### Requirement: Nix evidence core contract
r[mantle.release_provenance.nix_evidence_core.contract] Mantle MUST normalize Nix-related release evidence rows with store path ref, derivation identity, output name, realization role, caveat, artifact digest, and non-claim fields.

#### Scenario: Supported Nix evidence adapters pass
r[mantle.release_provenance.nix_evidence_core.fixtures.positive]
- GIVEN rows from Mantle build reports, release bundle rows, sidecar rows, Cairn Nix gates, Molten promotion evidence, and Valence provenance inputs
- WHEN Nix evidence validation runs
- THEN supported rows MUST pass with identity-only caveats and non-claims.

#### Scenario: Invalid Nix evidence fails closed
r[mantle.release_provenance.nix_evidence_core.fixtures.negative]
- GIVEN a row has malformed store path, wrong output, digest mismatch, unsupported derivation, missing caveats, ambiguous role, missing non-claims, or overclaims
- WHEN Nix evidence validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Nix evidence validation and adapters
r[mantle.release_provenance.nix_evidence_core.validation] Mantle MUST validate normalized Nix evidence in a pure core and keep build/evaluation shell work, file reads, and source adapter parsing outside that core.

#### Scenario: Adapter rows stay identity-only
r[mantle.release_provenance.nix_evidence_core.adapters]
- GIVEN an adapter normalizes Mantle build report, release provenance, Cairn Nix gate, Molten promotion, or Valence provenance material
- WHEN the normalized row is accepted
- THEN the accepted claim MUST remain limited to realization identity, digest shape, caveats, role, and non-claims.

### Requirement: Nix evidence documentation and final validation
r[mantle.release_provenance.nix_evidence_core.docs] Mantle operator docs MUST describe Nix evidence as realization identity only and name downstream migration boundaries.

#### Scenario: Final Nix evidence validation passes
r[mantle.release_provenance.nix_evidence_core.final_validation]
- GIVEN valid and invalid Nix evidence fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass, invalid fixtures MUST fail closed, Cairn validation/gates MUST pass, and documentation MUST preserve build/evaluation non-claims.

### Requirement: Release filesystem capability dependency
r[mantle.release_provenance.cap_std_boundary.dependency] Mantle MUST use `cap-std` only in crates or modules that own filesystem shell/adaptor behavior and MUST keep pure release planning cores free of ambient filesystem authority.

#### Scenario: Capability dependency is shell-scoped
- GIVEN release evidence code needs local filesystem access
- WHEN the access is added or reviewed
- THEN the `cap-std` dependency MUST remain in the Mantle binary shell surface rather than in no-std/pure release cores.

### Requirement: Release capability root wrappers
r[mantle.release_provenance.cap_std_boundary.root_wrappers] Mantle MUST expose typed capability roots for release evidence, witness rebuild, bootstrap, build artifact, and store roots.

#### Scenario: Relative paths authorize under declared roots
r[mantle.release_provenance.cap_std_boundary.tests.positive]
- GIVEN a request names a valid relative path and the required root authority is present
- WHEN the release path authorizer runs
- THEN it MUST return a validated relative path that can be used through the matching capability root.

### Requirement: Capability-relative release path conversion
r[mantle.release_provenance.cap_std_boundary.conversion] Mantle MUST convert targeted release path opens to capability-relative operations without moving filesystem authority into pure planning logic.

#### Scenario: Unsafe local paths fail closed
r[mantle.release_provenance.cap_std_boundary.tests.negative]
- GIVEN a request uses `../` traversal, an absolute path, missing root authority, wrong root authority, or a symlink escape
- WHEN capability boundary validation or shell read/write runs
- THEN the operation MUST fail before reading or writing outside the declared root.

### Requirement: Release capability boundary documentation and validation
r[mantle.release_provenance.cap_std_boundary.docs] Mantle docs MUST describe the local filesystem-authority boundary and preserve release-evidence non-claims.

#### Scenario: Capability validation evidence exists
r[mantle.release_provenance.cap_std_boundary.validation]
- GIVEN the cap-std release boundary change is complete
- WHEN focused validation runs
- THEN positive relative-path fixtures and negative traversal/absolute/missing-authority/symlink fixtures MUST pass, and Cairn validation/gates MUST pass before archive.

### Requirement: Function-address evidence policy

r[mantle.release_provenance.function_address_evidence.policy] Mantle release profiles SHOULD support disabled, optional, and required modes for function-address evidence. Required mode MUST fail closed when the function-address sidecar, Valence verification receipt or graph report, expected roles, expected schemas, claim scope, binary/source identity, BLAKE3 digests, or required non-claims are missing or invalid.

#### Scenario: Optional absent evidence is recorded
r[mantle.release_provenance.function_address_evidence.policy.optional_absent]
- GIVEN a release profile where function-address evidence is optional
- WHEN Mantle verifies a release bundle without function-address evidence
- THEN verification MUST record an absent or skipped disposition without claiming the bundle carries function-address evidence.

#### Scenario: Required evidence passes with matching sidecars
r[mantle.release_provenance.function_address_evidence.policy.required_valid]
- GIVEN a release profile where function-address evidence is required and the bundle carries matching sidecar, Valence receipt, binary identity, source identity, roles, schemas, and non-claims
- WHEN Mantle verifies the release bundle
- THEN verification MUST accept the function-address evidence contribution as bundle-local linkage.

#### Scenario: Required evidence missing fails
r[mantle.release_provenance.function_address_evidence.policy.required_missing]
- GIVEN a release profile where function-address evidence is required
- WHEN the bundle omits the function-address sidecar or Valence verification receipt
- THEN release verification MUST fail with deterministic diagnostics naming the missing evidence.

### Requirement: Function-address evidence binding

r[mantle.release_provenance.function_address_evidence.binding] Mantle MUST bind function-address evidence to release metadata by checking sidecar byte digest, Valence receipt or graph-report byte digest, Kamacite receipt identity when present, release binary identity, source artifact identity, external evidence role, schema, claim scope, and required non-claims.

#### Scenario: Sidecar and receipt digests are verified
r[mantle.release_provenance.function_address_evidence.binding.digests]
- GIVEN a release bundle declares function-address sidecar and Valence receipt artifacts
- WHEN Mantle verifies the bundle
- THEN the sidecar and receipt bytes MUST match the BLAKE3 digests recorded in release evidence metadata.

#### Scenario: Binary and source identities are linked
r[mantle.release_provenance.function_address_evidence.binding.binary_source]
- GIVEN function-address evidence is present in a release bundle
- WHEN Mantle verifies the bundle
- THEN the release binary identity and source artifact identity in bundle metadata MUST match the identities named by the function-address evidence metadata available to Mantle.

#### Scenario: Stale binding fails
r[mantle.release_provenance.function_address_evidence.binding.stale]
- GIVEN the sidecar bytes, Valence receipt bytes, role, schema, claim scope, binary identity, source identity, or non-claims do not match declared metadata
- WHEN Mantle verifies required function-address evidence
- THEN verification MUST fail with deterministic diagnostics naming the stale or mismatched field.

### Requirement: Function-address validation boundary

r[mantle.release_provenance.function_address_evidence.validation] Mantle MUST validate function-address release evidence as pure validation over loaded metadata rows while file reads, digest measurement, bundle assembly, and upstream evidence production remain outside the pure core.

#### Scenario: Mantle does not parse functions
r[mantle.release_provenance.function_address_evidence.validation.opaque]
- GIVEN a function-address sidecar contains individual function records
- WHEN Mantle verifies release evidence
- THEN Mantle MUST treat the sidecar as opaque external evidence and MUST NOT recompute or interpret individual function addresses.

#### Scenario: Boundary is visible in output
r[mantle.release_provenance.function_address_evidence.validation.boundary]
- GIVEN function-address release evidence passes
- WHEN Mantle reports the result
- THEN the supported claim MUST be limited to bundle-local path, digest, role, schema, claim-scope, binary/source identity, and non-claim validation.

### Requirement: Positive fixture coverage

r[mantle.release_provenance.function_address_evidence.fixtures.positive] Mantle MUST include positive fixtures for optional-absent, optional-present, and required-present function-address release evidence.

#### Scenario: Positive fixtures cover modes
r[mantle.release_provenance.function_address_evidence.fixtures.positive.modes]
- GIVEN optional absent, optional present, and required present release evidence fixtures
- WHEN the fixture suite runs
- THEN optional absent MUST record an absent disposition, optional present MUST verify bundle-local metadata, and required present MUST pass with matching Valence/Kamacite evidence metadata.

### Requirement: Negative fixture coverage

r[mantle.release_provenance.function_address_evidence.fixtures.negative] Mantle MUST include fail-closed fixtures for missing sidecars, missing Valence receipts, stale BLAKE3 digests, wrong roles, wrong schemas, unsupported claim scopes, binary identity mismatch, source identity mismatch, weakened non-claims, and overclaims.

#### Scenario: Negative fixtures fail required mode
r[mantle.release_provenance.function_address_evidence.fixtures.negative.required]
- GIVEN invalid required-mode function-address release evidence fixtures
- WHEN release verification evaluates the fixtures
- THEN each fixture MUST fail closed with deterministic diagnostics naming the invalid evidence class.

### Requirement: Function-address evidence documentation

r[mantle.release_provenance.function_address_evidence.docs] Mantle operator documentation MUST distinguish optional generic release evidence from required Onix stack release evidence and MUST preserve the opaque external-evidence boundary.

#### Scenario: Docs preserve ownership boundary
r[mantle.release_provenance.function_address_evidence.docs.boundary]
- GIVEN an operator reads function-address release evidence documentation
- WHEN the evidence flow is described
- THEN the docs MUST state that Octet owns Rust extraction, Kamacite owns portable receipts, Valence owns linkage semantics, Mantle owns bundle-local binding, and Cairn owns lifecycle/readiness policy.

### Requirement: Final validation evidence

r[mantle.release_provenance.function_address_evidence.final_validation] The change MUST include positive and negative release fixtures, focused release-provenance tests, constants/profile checks, Cairn validation, and proposal/design/tasks gates before archive.

#### Scenario: Validation covers pass and fail cases
r[mantle.release_provenance.function_address_evidence.final_validation.fixtures]
- GIVEN valid and invalid function-address release evidence fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass, invalid fixtures MUST fail closed, and saved evidence MUST bind profile and input identities.

