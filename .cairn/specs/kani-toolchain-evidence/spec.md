# Kani Toolchain Evidence Specification

## Purpose

Defines the `kani-toolchain-evidence` capability.

## Requirements

### Requirement: Kani toolchain identity
r[mantle.kani_toolchain_evidence.identity] Mantle MUST record Kani verifier toolchain identity for release evidence, including Kani, Rust, CBMC, solver, invocation wrapper, closure identity, and receipt digest.

#### Scenario: Complete Kani toolchain identity passes
r[mantle.kani_toolchain_evidence.identity.complete]
- GIVEN release evidence includes all required Kani toolchain identity fields
- WHEN Mantle verifies the bundle
- THEN the fields MUST be preserved as deterministic release evidence metadata.

#### Scenario: Missing Kani identity fails
r[mantle.kani_toolchain_evidence.identity.missing]
- GIVEN release evidence omits required Kani, CBMC, solver, wrapper, closure, or receipt identity
- WHEN Mantle verifies the bundle
- THEN verification MUST fail closed.

### Requirement: Kani bundle linkage
r[mantle.kani_toolchain_evidence.bundle_linkage] Mantle MUST link Kani receipt artifacts and verifier toolchain identity into release evidence bundles by digest and role.

#### Scenario: Matching receipt digest passes
r[mantle.kani_toolchain_evidence.bundle_linkage.match]
- GIVEN a release bundle references a Kani receipt digest that matches the included artifact
- WHEN Mantle verifies the bundle
- THEN the Kani evidence linkage MUST pass.

#### Scenario: Receipt digest mismatch fails
r[mantle.kani_toolchain_evidence.bundle_linkage.mismatch]
- GIVEN a release bundle references a Kani receipt digest that does not match the included artifact
- WHEN Mantle verifies the bundle
- THEN verification MUST fail with a deterministic digest mismatch diagnostic.

### Requirement: Valence-owned Kani semantics boundary
r[mantle.kani_toolchain_evidence.valence_boundary] Mantle MUST treat Kani receipts as external evidence whose semantic validation is Valence-owned, while Mantle verifies bundle identity, role, digest, and non-claim preservation.

#### Scenario: Mantle preserves external role
r[mantle.kani_toolchain_evidence.valence_boundary.external]
- GIVEN a bundle includes a Kani receipt already validated by Valence
- WHEN Mantle verifies release evidence
- THEN Mantle MUST preserve the external evidence role without reinterpreting Kani semantics.

#### Scenario: Semantic promotion is rejected
r[mantle.kani_toolchain_evidence.valence_boundary.no_promotion]
- GIVEN a bundle tries to promote Kani receipt semantics beyond the Valence-validated role
- WHEN Mantle verifies the bundle
- THEN verification MUST reject the promotion.

### Requirement: Kani toolchain positive fixtures
r[mantle.kani_toolchain_evidence.positive_fixtures] Mantle MUST include positive fixtures for release bundles with matching Kani receipt and toolchain identity.

#### Scenario: Positive Kani bundle fixture passes
r[mantle.kani_toolchain_evidence.positive_fixtures.valid]
- GIVEN a release bundle fixture has matching Kani receipt and toolchain identity
- WHEN bundle verification runs
- THEN verification MUST pass deterministically.

### Requirement: Kani toolchain negative fixtures
r[mantle.kani_toolchain_evidence.negative_fixtures] Mantle MUST include negative fixtures for missing Kani version, stale closure identity, unsupported solver metadata, mismatched receipt digest, and missing non-claims.

#### Scenario: Stale closure fails
r[mantle.kani_toolchain_evidence.negative_fixtures.stale]
- GIVEN Kani toolchain metadata names a stale or mismatched closure identity
- WHEN Mantle verifies release evidence
- THEN verification MUST fail closed.

#### Scenario: Unsupported solver metadata fails
r[mantle.kani_toolchain_evidence.negative_fixtures.solver]
- GIVEN Kani toolchain metadata names unsupported or malformed solver identity
- WHEN Mantle verifies release evidence
- THEN verification MUST fail closed.

### Requirement: Kani toolchain non-claims
r[mantle.kani_toolchain_evidence.non_claims] Mantle MUST preserve non-claims that Kani release evidence does not prove verifier soundness, Kani semantics, whole-program correctness, or release eligibility by itself.

#### Scenario: Missing non-claims fail
r[mantle.kani_toolchain_evidence.non_claims.missing]
- GIVEN a release bundle includes Kani evidence without required non-claims
- WHEN Mantle verifies the bundle
- THEN verification MUST fail closed.

### Requirement: Kani release-evidence operator documentation
r[mantle.kani_toolchain_evidence.operator_docs] Mantle documentation MUST explain how Kani receipts and toolchain identities are packaged or referenced as release evidence.

#### Scenario: Operator docs explain Kani evidence bundle
r[mantle.kani_toolchain_evidence.operator_docs.visible]
- GIVEN an operator reads release-evidence documentation
- WHEN Kani evidence is described
- THEN the documentation MUST name the receipt, toolchain identity, Valence semantic boundary, and Mantle non-claims.
