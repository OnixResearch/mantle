## Why

crunch already knows more about a build than most SBOM tools can recover:
logical store paths, derivation edges, runtime references, fixed-output
hashes, lockfile source and patch provenance, and final output content
hashes. If we bolt a standards-format SBOM exporter on top, we lose
structure and inherit someone else's model.

The stronger direction is a crunch-native attestation layer: a
deterministic, hash-addressed record of what was declared, what crunch
observed, and how a closure or project view was assembled. That fits the
project's goals better than SPDX/CycloneDX interoperability.

For this change, **attestation** means the canonical persisted object.
**Provenance** means the claims and facts carried inside that object.

This change writes down that direction before implementation starts. It gives
crunch a first-class attestation feature instead of an afterthought report.

## What Changes

- **Native attestation model**: define a crunch-owned schema with distinct
  node and edge kinds for sources, recipes, artifacts, patches, projects,
  and closures.
- **Deterministic attestation digests**: require canonical normalization and
  BLAKE3 digests for per-artifact, per-closure, and per-project
  attestations.
- **Claims vs facts split**: keep user-declared provenance claims separate
  from crunch-observed build/store facts.
- **First-class build integration**: require attestation generation and
  persistence as part of successful builds and substitutions.
- **Operator surface**: add CLI commands to inspect, verify, diff, and
  assemble artifact, closure, and project attestations.
- **Nickel package metadata**: let builder-layer package definitions attach
  structured provenance claims without changing derivation hashes.

## Capabilities

### New Capabilities
- `native-attestation-model`: canonical crunch-native attestation objects
- `artifact-attestation`: per-output attestation bound to content hashes
- `closure-attestation`: closure-level attestation assembled from member
  artifact attestations and typed edges
- `project-attestation`: project-level attestation assembled from manifest,
  lockfile, patches, and selected built roots
- `attestation-verify`: local verification of canonical digests and graph
  integrity

### Modified Capabilities
- `crunch build`: persists native attestations for successful outputs
- `builders.mkDerivation`: accepts optional provenance claims metadata
- `StoreHandle`: stores and retrieves attestations keyed by logical store
  path identity or rooted closure selection
- `crunch --json build`: reports generated attestation references

## Impact

- **Files**: new attestation crate(s), build/store/CLI wiring, builder-layer
  Nickel contracts, and project-attestation assembly
- **APIs**: new native attestation types, canonical serialization rules, and
  store accessors for artifact, closure, and project attestations
- **Dependencies**: no standards-format dependency required; reuse BLAKE3 and
  existing graph/store primitives
- **Testing**: deterministic serialization tests, closure-order invariance
  tests, build/substitution integration tests, and verification tests
- **Non-goals**: SPDX/CycloneDX interoperability, license scanning, CVE
  matching, or making user-authored provenance claims affect derivation hashes
