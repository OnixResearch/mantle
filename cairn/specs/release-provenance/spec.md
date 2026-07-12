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

### Requirement: Release tree discovery does not follow symlinks

r[mantle.release_provenance.bundle_tree_copy.no_follow] Mantle MUST classify release tree entries with no-follow metadata and MUST NOT recurse through a symlink during discovery, copying, hashing, or verification, even when the symlink target is a directory.

#### Scenario: Internal relative symlink is copied as one entry

r[mantle.release_provenance.bundle_tree_copy.fixtures.positive]
- GIVEN a release input tree contains regular nested entries and a supported relative symlink whose target stays within the planned tree
- WHEN Mantle plans and copies the tree
- THEN the symlink MUST be represented and hashed as a symlink entry
- AND traversal MUST NOT enumerate descendants through the link.

### Requirement: Tree copy planning is pure and deterministic

r[mantle.release_provenance.bundle_tree_copy.plan] Mantle MUST build a deterministic copy plan from normalized relative paths, no-follow entry kinds, modes, and symlink targets before mutation, while directory enumeration, byte reads/writes, capability handling, and metadata revalidation remain in the shell.

#### Scenario: Invalid tree shape blocks mutation

r[mantle.release_provenance.bundle_tree_copy.plan.invalid]
- GIVEN normalized observations contain an absolute path, parent traversal, duplicate path, missing real-directory parent, unsupported special file, excessive bound, or invalid symlink target
- WHEN the pure planner evaluates the tree
- THEN it MUST return ordered blockers
- AND no destination mutation MAY begin.

### Requirement: Destination writes remain capability-confined

r[mantle.release_provenance.bundle_tree_copy.destination_confinement] Mantle MUST execute every tree-copy mutation relative to the declared destination capability without following a destination symlink, and MUST revalidate source kind and destination parent kind before each operation.

#### Scenario: Directory symlink escape fails without external writes

r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
- GIVEN a source tree contains a directory symlink whose followed descendants would map through a destination symlink to an external target containing a sentinel
- WHEN release bundle creation processes the tree
- THEN Mantle MUST fail closed before writing any descendant through the link
- AND the external sentinel and every path outside the destination capability MUST remain unchanged.

#### Scenario: Destination or source type drift fails closed

r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
- GIVEN a planned source entry or destination parent changes type, becomes a symlink, or no longer matches its no-follow observation before execution
- WHEN the shell revalidates the operation
- THEN Mantle MUST stop with a deterministic drift diagnostic
- AND it MUST NOT follow the changed entry.

### Requirement: Supported symlinks remain inside the planned tree

r[mantle.release_provenance.bundle_tree_copy.symlink_policy] Mantle MAY preserve a symlink only when its target is relative, lexically resolves within the planned tree from the link's parent, names a planned entry, and is copied without traversal; absolute, escaping, unplanned, or unsupported symlinks MUST fail closed.

#### Scenario: Escaping target is rejected

r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.target]
- GIVEN a release tree contains an absolute symlink target or a relative target that normalizes outside the planned root
- WHEN tree-copy planning runs
- THEN the plan MUST be rejected with a symlink-target diagnostic
- AND no bundle or external target MAY be mutated.

### Requirement: Confinement regression evidence is required

r[mantle.release_provenance.bundle_tree_copy.validation] The change MUST include positive nested-tree/internal-symlink tests and negative source-link, destination-link, target-escape, type-drift, special-file, and external-sentinel tests through the production release bundle copy path.

#### Scenario: Production regression matrix proves confinement

r[mantle.release_provenance.bundle_tree_copy.validation.production]
- GIVEN the positive and negative copy fixtures
- WHEN focused release evidence tests run
- THEN valid trees MUST preserve deterministic BLAKE3 identities
- AND every escape fixture MUST fail while all out-of-root sentinels remain unchanged.

### Requirement: Mantle supports generic opaque evidence sidecars

r[mantle.release_provenance.opaque_evidence_sidecar_binding.contract] Mantle MUST support a generic release-evidence sidecar binding contract with canonical envelope hash, evidence kind, profile version, upstream validation hash, source artifact hash, release binary hash, policy hashes, claim scope, projections, and non-claims.

#### Scenario: Function-address uses generic sidecar binding

r[mantle.release_provenance.opaque_evidence_sidecar_binding.function_address]
- GIVEN function-address evidence is available as a canonical evidence-chain envelope and Valence validation receipt
- WHEN Mantle binds it to release artifacts
- THEN Mantle MUST use the generic opaque evidence sidecar binding path.

### Requirement: Release binding commits to typed links

r[mantle.release_provenance.opaque_evidence_sidecar_binding.links] Mantle MUST domain-separate sidecar identity, upstream validation identity, source artifact identity, release binary identity, and policy identity in release binding receipts.

#### Scenario: Source and binary linkage is checked

r[mantle.release_provenance.opaque_evidence_sidecar_binding.positive]
- GIVEN a valid sidecar binding references matching source and release binary artifacts
- WHEN release validation runs
- THEN Mantle MUST emit a deterministic binding receipt that commits to those typed links.

### Requirement: Evidence payloads remain opaque to Mantle core

r[mantle.release_provenance.opaque_evidence_sidecar_binding.opaque_core] Mantle release validation core MUST NOT parse profile payloads, Rust source, proof terms, or Preserves internals for opaque evidence sidecars.

#### Scenario: Core validates metadata only

r[mantle.release_provenance.opaque_evidence_sidecar_binding.validation]
- GIVEN the CLI has loaded and hashed sidecar metadata
- WHEN release validation core runs
- THEN the core MUST evaluate typed in-memory metadata and deterministic diagnostics without filesystem or parser effects.

### Requirement: Invalid sidecar bindings fail closed

r[mantle.release_provenance.opaque_evidence_sidecar_binding.negative] Mantle MUST reject or mark invalid opaque sidecar bindings with unknown evidence kind, missing canonical hash, stale upstream validation hash, source/binary mismatch, unsupported claim scope, wrong role/schema, projection drift, malformed BLAKE3, or weakened non-claims.

#### Scenario: Non-function-address stub proves generality

- GIVEN a supported non-function-address evidence kind has a canonical envelope and upstream validation receipt
- WHEN Mantle validates release binding metadata
- THEN Mantle MUST bind it through the same generic sidecar path without adding profile-specific release code.

### Requirement: Release verification produces one final policy decision

r[mantle.release_provenance.verification_decision.complete] Mantle MUST aggregate manifest integrity and every selected release policy contribution into one final deterministic decision before any success output is rendered.

#### Scenario: All selected checks pass

r[mantle.release_provenance.verification_decision.fixtures.positive]
- GIVEN manifest integrity and every optional or required contribution selected by the invocation satisfy policy
- WHEN release verification aggregates the normalized results
- THEN the final decision MUST be valid
- AND it MUST preserve each bounded contributor result without strengthening its claim.

#### Scenario: A late required check rejects the release

r[mantle.release_provenance.verification_decision.fixtures.negative]
- GIVEN manifest integrity passes but required reproducibility, deterministic proof, provider fixed-point proof, stack provenance, external role, StageX, function-address, or Cairn handoff policy is unsatisfied
- WHEN release verification aggregates the normalized results
- THEN the final decision MUST be invalid with ordered diagnostics naming every safely evaluable blocker
- AND manifest integrity success MUST NOT override the rejection.

### Requirement: Verification aggregation preserves core and shell boundaries

r[mantle.release_provenance.verification_decision.boundary] Mantle MUST compute final validity, disposition, contributor status, and ordered policy diagnostics in a pure core over supplied facts, while filesystem reads, specialized evaluator I/O, serialization, stdout/stderr, and process exit remain in the CLI shell.

#### Scenario: Decision is testable without CLI effects

r[mantle.release_provenance.verification_decision.boundary.test]
- GIVEN in-memory policy requirements and normalized contributor results
- WHEN the decision core evaluates them
- THEN it MUST return the same final decision independent of filesystem state, environment, clocks, output streams, or process state.

### Requirement: Required contributors cannot bypass the final decision

r[mantle.release_provenance.verification_decision.completeness] Every release policy contributor that can block an invocation MUST have an explicit mapping into the final decision, and adding a contributor without decision coverage MUST fail a compile-time exhaustive match or a focused completeness check.

#### Scenario: New evidence policy is wired into aggregation

r[mantle.release_provenance.verification_decision.completeness.test]
- GIVEN a release profile adds a required evidence contributor
- WHEN verification decision coverage is checked
- THEN the contributor MUST affect final validity and appear in ordered checks
- AND it MUST NOT be evaluated only after rendering or omitted from the decision.

### Requirement: Function-address binding receipt schema is versioned
r[mantle.release_provenance.function_address_binding_schema.contract] Mantle MUST define a versioned function-address release binding receipt schema for Cairn readiness consumption.

#### Scenario: Binding receipt has required fields
r[mantle.release_provenance.function_address_binding_schema.positive]
- GIVEN Mantle validates function-address release evidence successfully
- WHEN it renders the binding receipt
- THEN the receipt MUST include validity, verdict, receipt hash, claim scope, sidecar digest, Valence receipt digest, source archive digest, release binary digest, optional Kamacite digest, and non-claim boundaries.

### Requirement: Binding receipt maps to Cairn readiness input
r[mantle.release_provenance.function_address_binding_schema.cairn_mapping] Mantle SHOULD document how each binding receipt field maps to Cairn function-address readiness validation.

#### Scenario: Cairn extracts fields without wrapper glue
r[mantle.release_provenance.function_address_binding_schema.validation]
- GIVEN a Mantle function-address binding receipt follows the versioned schema
- WHEN Cairn release-readiness consumes it
- THEN Cairn SHOULD read the receipt directly without a smoke-specific wrapper.

### Requirement: Malformed binding receipts fail closed
r[mantle.release_provenance.function_address_binding_schema.negative] Mantle MUST reject or mark invalid binding receipts with missing required fields, malformed BLAKE3 hashes, stale Valence/Kamacite digests, unsupported claim scope, or weakened non-claim boundaries.

#### Scenario: Stable renderer preserves release verification
r[mantle.release_provenance.function_address_binding_schema.render]
- GIVEN Mantle renders a binding receipt
- WHEN an operator inspects it
- THEN the receipt MUST preserve the underlying release verification disposition and diagnostics without adding semantic correctness claims.

### Requirement: CLI binds function-address evidence to release artifacts
r[mantle.release_provenance.function_address_binding_cli.command] Mantle MUST provide a CLI command that binds Valence/Kamacite function-address evidence to release source and binary artifact identities.

#### Scenario: Required binding passes
r[mantle.release_provenance.function_address_binding_cli.positive]
- GIVEN a release manifest has matching source archive and release binary digests, Valence function-address evidence, and Kamacite receipt metadata
- WHEN Mantle validates function-address release evidence in required mode
- THEN the CLI MUST emit a passing binding receipt.

#### Scenario: Logical and artifact receipt identities stay distinct
r[mantle.release_provenance.function_address_binding_cli.receipt.identity_domains]
- GIVEN a bundled Valence or Kamacite JSON artifact has a file-byte BLAKE3 digest and exposes a distinct logical `receipt_hash`
- WHEN Mantle renders the Cairn-facing binding
- THEN the legacy top-level receipt digest field MUST carry the upstream logical identity while the embedded verification summary retains the bundled artifact-byte digest.

### Requirement: Binding CLI shell owns filesystem effects
r[mantle.release_provenance.function_address_binding_cli.shell] Mantle CLI file reads, path resolution, stdout/stderr, and receipt writes MUST remain outside `crunch-release-core` validation logic.

#### Scenario: Core receives typed release evidence
r[mantle.release_provenance.function_address_binding_cli.receipt]
- GIVEN the CLI has loaded release artifact metadata and external evidence metadata
- WHEN validation runs
- THEN `crunch-release-core` MUST evaluate typed in-memory release evidence and return deterministic diagnostics.

#### Scenario: Replaced receipt bytes fail before output
r[mantle.release_provenance.function_address_binding_cli.shell.replacement]
- GIVEN bundle verification completed but a selected receipt file is replaced before the CLI identity read
- WHEN the shell reopens that path
- THEN the reopened bytes MUST match the verified manifest BLAKE3 digest or the command MUST fail before writing a binding receipt.

### Requirement: Invalid binding fails closed
r[mantle.release_provenance.function_address_binding_cli.negative] Mantle MUST reject function-address binding inputs with missing evidence, stale digest, wrong role, wrong schema, unsupported claim scope, source/binary mismatch, or overclaiming non-claim text.

#### Scenario: Stack smoke uses Mantle-owned receipt
r[mantle.release_provenance.function_address_binding_cli.validation]
- GIVEN Octet, Kamacite, and Valence produce function-address evidence
- WHEN Mantle binds the evidence to release artifacts
- THEN the binding receipt MUST remain identity/linkage-only and suitable for Cairn readiness input.

### Requirement: Mantle binds Preserves receipt sidecar metadata
r[mantle.release_provenance.function_address_preserves_sidecars.contract] Mantle MUST support function-address release evidence metadata for canonical Kamacite Preserves receipt sidecars, including role, schema version, canonical BLAKE3 receipt hash, bounded byte size, source archive digest, release binary digest, Valence artifact and logical receipt hashes, claim scope, and non-claims.

#### Scenario: Required Preserves-backed binding passes
r[mantle.release_provenance.function_address_preserves_sidecars.positive]
- GIVEN release evidence includes matching source archive, release binary, Valence receipt, and canonical Kamacite Preserves receipt metadata
- WHEN Mantle validates function-address release evidence in required mode
- THEN Mantle MUST emit a passing release binding receipt.

#### Scenario: Manifest-driven CLI rejects mixed-time identity drift
- GIVEN a verified release manifest contains exactly one typed `function-address-preserves-v1` binding
- WHEN `mantle release function-address-bind --from-preserves-binding` reopens its sidecars
- THEN Mantle MUST perform bounded no-follow reads, compare reopened byte digests and canonical Preserves size with the manifest, and fail before output on drift.

### Requirement: Mantle treats Preserves receipts as opaque evidence
r[mantle.release_provenance.function_address_preserves_sidecars.opaque] Mantle MUST NOT parse Rust functions or reinterpret Preserves receipt internals when binding function-address release evidence.

#### Scenario: Core validates typed metadata only
r[mantle.release_provenance.function_address_preserves_sidecars.validation]
- GIVEN the CLI has loaded and hashed sidecar artifacts
- WHEN release validation core runs
- THEN the core MUST evaluate typed metadata and deterministic diagnostics without filesystem reads or Preserves parser effects.

### Requirement: Compatibility projections are bound to canonical identity
r[mantle.release_provenance.function_address_preserves_sidecars.json_projection] Mantle MAY accept JSON compatibility sidecars only when their distinct artifact-byte digest and public logical `receipt_hash` bind to the canonical Kamacite Preserves receipt identity.

#### Scenario: Invalid sidecar binding fails closed
r[mantle.release_provenance.function_address_preserves_sidecars.negative]
- GIVEN function-address release evidence has missing or stale Preserves hash, stale Valence hash, projection drift, wrong role/schema, unsupported claim scope, malformed BLAKE3, source/binary mismatch, or overclaiming text
- WHEN Mantle validates the binding
- THEN validation MUST fail with deterministic release-evidence diagnostics.

### Requirement: Release bundle publication is atomic

r[mantle.release_provenance.bundle_publication.atomic_commit] Mantle MUST assemble a release bundle in a private sibling staging directory and make it visible at the final path only through one same-filesystem atomic no-clobber rename after successful verification.

#### Scenario: Complete staged bundle is published

r[mantle.release_provenance.bundle_publication.fixtures.positive]
- GIVEN all declared inputs are valid, the final destination is absent, and staging verification passes
- WHEN release bundle creation commits
- THEN the final path MUST appear as one complete canonical bundle
- AND CLI success MUST be emitted only after the atomic rename succeeds.

#### Scenario: Concurrent destination creation does not clobber

r[mantle.release_provenance.bundle_publication.fixtures.negative.race]
- GIVEN another process creates a file, symlink, or directory at the final destination after planning and before commit
- WHEN Mantle attempts publication
- THEN the no-clobber commit MUST fail
- AND the competing destination MUST remain unchanged.

### Requirement: Staging is complete and verified before commit

r[mantle.release_provenance.bundle_publication.staging_validation] Mantle MUST write the canonical manifest after all planned artifacts are staged and MUST run the production bundle verifier against the staging root before the stage becomes commit-eligible.

#### Scenario: Invalid staged artifact prevents publication

r[mantle.release_provenance.bundle_publication.fixtures.negative.verification]
- GIVEN a staged artifact is missing, stale, malformed, noncanonical, or inconsistent with proof or external-evidence linkage
- WHEN staging verification runs
- THEN commit eligibility MUST be denied with a deterministic phase diagnostic
- AND the final destination MUST remain absent.

### Requirement: Failed assembly does not poison the final path

r[mantle.release_provenance.bundle_publication.failure_isolation] Any pre-commit copy, hash, serialization, policy, verification, or cleanup error MUST leave an existing final destination untouched or an absent final destination absent; partial artifacts MUST remain confined to a Mantle-owned stage.

#### Scenario: Mid-assembly failure is retryable

r[mantle.release_provenance.bundle_publication.retry]
- GIVEN a named assembly phase fails after one or more artifacts have been staged
- WHEN the command returns and the operator retries with corrected inputs
- THEN the failed attempt MUST NOT leave a partial final bundle
- AND a fresh attempt MUST be able to stage, verify, and publish without manual deletion of final-path contents.

#### Scenario: Stale stage cleanup is ownership-bounded

r[mantle.release_provenance.bundle_publication.stale_stage]
- GIVEN a sibling staging directory remains after interruption
- WHEN retry or cleanup evaluates it
- THEN Mantle MAY remove or quarantine it only when a valid ownership marker and matching plan identity prove it is Mantle-owned staging state
- AND an unrecognized sibling path MUST remain untouched.

### Requirement: Atomic publication planning is pure

r[mantle.release_provenance.bundle_publication.boundary] Mantle MUST compute artifact layout, destination eligibility, deterministic assembly order, state transitions, and commit eligibility in a pure core, while filesystem observation, capability setup, staging writes, verification I/O, rename, cleanup, and rendering remain in the shell.

#### Scenario: Publication state machine is deterministic

r[mantle.release_provenance.bundle_publication.boundary.test]
- GIVEN normalized input facts, destination observations, assembly events, and verification results
- WHEN the publication core evaluates them
- THEN it MUST return deterministic next states and blockers without filesystem, environment, clock, process, or output effects.

### Requirement: Publication regression evidence covers failures

r[mantle.release_provenance.bundle_publication.validation] The change MUST test successful publication and retry plus injected copy, hash, manifest, verification, cleanup, destination-symlink, and commit-race failures through the production release creation path.

#### Scenario: Readers never observe a partial final bundle

r[mantle.release_provenance.bundle_publication.validation.visibility]
- GIVEN production creation is observed at every named assembly and commit phase
- WHEN positive and injected-failure fixtures run
- THEN the final path MUST be absent before commit and complete after commit
- AND no failure fixture MAY expose a partial final manifest or artifact tree.

### Requirement: Deterministic release admission requires genuine rebuild evidence

r[mantle.release_provenance.deterministic_rebuild_admission.contract] `mantle release verify` MUST treat matching output digests as insufficient for deterministic-release admission unless the proof receipt binds an accepted rebuild descriptor and every run proves target-byte exclusion, declared input authority, fresh roots, supported sandbox evidence, and matching selected outputs.

#### Scenario: Legacy path-bound receipt is non-promoting

r[mantle.release_provenance.deterministic_rebuild_admission.legacy]
- GIVEN a deterministic proof receipt records matching run digests but lacks content-bound recipe/tool identities or target-authority evidence
- WHEN release verification evaluates a required deterministic-release policy
- THEN verification MUST return a non-promoting missing-genuine-rebuild-evidence disposition
- AND it MUST NOT report the release as deterministic or `self-rebuild-match` eligible.

### Requirement: Determinism consumers apply one admission rule

r[mantle.release_provenance.deterministic_rebuild_admission.validation] The production real-release rail, deterministic receipt checker, summary renderer, bootstrap-parity consumer, and release verifier MUST apply the same genuine-rebuild admission contract and MUST bind the accepted rebuild descriptor BLAKE3 in their evidence.

#### Scenario: Copy-only production rail is rejected

r[mantle.release_provenance.deterministic_rebuild_admission.fixtures.negative]
- GIVEN the production rail writes an output by copying bytes from `MANTLE_REPRODUCE_BUNDLE_DIR` or another published-target alias
- WHEN the receipt checker, summary renderer, bootstrap-parity consumer, or release verifier evaluates the evidence
- THEN every consumer MUST reject promotion with a deterministic blocker
- AND no success text or receipt MAY claim that recorded source inputs rebuilt the artifact.
