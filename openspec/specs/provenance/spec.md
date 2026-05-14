# Provenance Specification

## Purpose

Defines Mantle's native attestation model, canonical digest rules, and the
retrieval semantics for artifact, closure, and project provenance records.

## Requirements

### Requirement: Native attestation model

The system MUST define a crunch-native attestation model as a first-class
feature. The native model MUST represent mantle concepts directly instead of
translating them into an external SBOM schema.

At minimum the native model MUST support distinct node kinds for:
- source inputs
- build recipes
- output artifacts
- patches
- projects
- closures

At minimum the native model MUST support distinct edge kinds for:
- build-input
- runtime-reference
- produced-by
- fetched-from
- patched-by
- member-of-closure
- declared-by-project

#### Scenario: Artifact attestation records native node and edge kinds

- GIVEN a successful build of a derivation with fetched sources and runtime references
- WHEN mantle materializes its artifact attestation
- THEN the record identifies the output as an artifact node
- AND it records the producing recipe node separately
- AND it records fetched source edges separately from runtime-reference edges

### Requirement: Claims and facts stay distinct

The system MUST preserve declared claims separately from observed facts inside
each attestation object.

Declared claims include package-author metadata such as name, version,
supplier, homepage, and license. Observed facts include recipe identity,
logical store path, output name, runtime references, lockfile-resolved source
provenance, patch facts, and final content hashes.

#### Scenario: User claim and observed fact both appear without conflation

- GIVEN a package definition that declares a supplier and homepage
- AND a successful build that produces a store path and runtime references
- WHEN mantle materializes the artifact attestation
- THEN the supplier and homepage appear in claims
- AND the store path and runtime references appear in facts
- AND the attestation does not collapse those sections into one untyped map

### Requirement: Canonical deterministic attestation digest

The system MUST define one canonical normalization and serialization process
for native attestations. Given the same normalized claims, facts, roots, and
schema version, the canonical bytes and BLAKE3 digest MUST be identical.

Traversal order, hash-map insertion order, and closure discovery order MUST
NOT affect the canonical bytes or digest.

#### Scenario: Traversal order does not change closure digest

- GIVEN two closure walks that discover the same members and typed edges in different orders
- WHEN mantle canonicalizes the closure attestation
- THEN the canonical bytes are identical
- AND the BLAKE3 closure digest is identical

### Requirement: Artifact attestations are the unit of persistence

The system MUST persist per-output artifact attestations keyed by logical
store-path identity. Closure and project attestations MUST compose from those
artifact attestations rather than repeating the full artifact payload inline.

#### Scenario: Closure attestation composes from stored artifact attestations

- GIVEN two artifact attestations already persisted for outputs A and B
- WHEN mantle materializes a closure attestation rooted at A
- THEN the closure attestation references the member artifact attestations
- AND the closure attestation digest is computed from member digests plus typed edges
- AND mantle does not need to duplicate the full payload of A and B to identify the closure

### Requirement: Attestation retrieval is keyed by native identity

The system MUST support retrieving persisted artifact attestations by logical
store path identity. Aggregate closure and project attestations MUST be
retrievable by their rooted or project-scoped selection identity.

#### Scenario: Artifact attestation retrieved by logical store path

- GIVEN a persisted artifact attestation for logical store path A
- WHEN mantle requests the attestation for A from the store layer
- THEN the matching artifact attestation is returned
- AND the lookup does not depend on a host-specific exported filesystem path

#### Scenario: Closure attestation retrieved by rooted selection

- GIVEN a persisted runtime closure attestation rooted at artifacts A and B
- WHEN mantle requests that same rooted closure selection
- THEN the matching closure attestation is returned
- AND a different root set or closure policy does not alias the same retrieval key

### Requirement: Attestations are generated for successful builds and substitutions

The system MUST generate or retrieve artifact attestations for every
successful build output and every accepted substitution result.

If a substituted path does not come with a remote attestation, mantle MUST
synthesize the observed-facts portion locally from the accepted store data
before reporting success.

#### Scenario: Successful local build persists artifact attestation

- GIVEN a derivation that builds successfully
- WHEN mantle persists its final store metadata
- THEN an artifact attestation is persisted for each successful output
- AND the artifact attestation is bound to the final content hash and logical store path

#### Scenario: Accepted substitution yields artifact attestation

- GIVEN a remote cache hit that passes mantle's acceptance checks
- WHEN the substituted output is accepted locally
- THEN mantle fetches or synthesizes a native artifact attestation for that output
- AND later closure assembly can use that artifact attestation without special casing substitutions

### Requirement: Closure attestations are root-exact

The system MUST support closure attestations rooted at one or more selected
artifacts. A closure attestation MUST include exactly the reachable members and
selected typed edges for the configured closure semantics.

The root selection and closure semantics MUST be part of the closure
attestation input so different closure policies produce different digests.

#### Scenario: Runtime closure excludes unrelated build-only artifact

- GIVEN artifact A has a runtime-reference edge to B
- AND artifact A has a historical build-only relationship to C that is not part of runtime closure semantics
- WHEN mantle materializes a runtime closure attestation rooted at A
- THEN B is included in the closure members
- AND C is excluded from the runtime closure members
- AND the closure digest reflects that root and edge-policy choice

### Requirement: Project attestations capture project-scoped provenance

The system MUST support a project attestation assembled from the project
manifest, lockfile, resolved source and patch facts, and selected built roots.

A project attestation MUST identify the project scope separately from any one
artifact or closure attestation.

#### Scenario: Project attestation records locked sources and selected roots

- GIVEN a project with `mantle-project.ncl`, `mantle.lock`, and built roots A and B
- WHEN mantle materializes the project attestation
- THEN it records the project node separately from artifacts A and B
- AND it records the lockfile-resolved source and patch facts in the project scope
- AND it records that A and B are the selected built roots for that project attestation
