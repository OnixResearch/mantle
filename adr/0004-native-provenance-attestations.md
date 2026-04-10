# ADR 0004: Native Attestations

## Status

Proposed (2026-04-09)

## Context

crunch already computes most of the data needed for a strong supply-chain
record:

- evaluated derivation intent
- project lockfile source, mirror, and patch provenance
- final output identity (`PathInfo`, runtime references, deriver)
- BLAKE3-based artifact and store identity

The tempting next step is "add SBOM support", but standard SBOM formats do
not fit crunch well. They flatten distinct edge kinds, blur author claims with
observed facts, and force crunch-native identities into weaker package/document
conventions.

crunch is not trying to integrate with existing enterprise SBOM pipelines
right now. The goal is a first-class, deterministic, crunch-native attestation
feature that matches the store model and can eventually be verified.

## Decision

crunch will define a native attestation format instead of using SPDX or
CycloneDX as its source of truth.

Key properties:

- **Native concepts**: attestation objects model sources, recipes, artifacts,
  patches, projects, and closures directly.
- **Typed edges**: build-input, runtime-reference, produced-by,
  fetched-from, patched-by, member-of-closure, declared-by-project.
- **Claims vs facts split**: user-declared metadata stays separate from
  crunch-observed build/store facts.
- **Artifact-first persistence**: per-output artifact attestations are the
  storage unit; closure/project attestations compose from them.
- **Canonical deterministic digest**: the canonical native representation is
  hashed with BLAKE3. Closure digests are Merkle-style summaries over member
  artifact-attestation digests plus typed edges.
- **Builder-layer metadata**: optional provenance claims live in the builder
  package layer, not the minimal core derivation contract, and do not affect
  derivation hashes by default.

## Consequences

- crunch gets an attestation system that matches its own graph and store model
  instead of a lossy export-first design.
- The canonical native format can be made deterministic and easier to verify
  than a standards-based export pipeline.
- Existing external SBOM tools will not consume the native format directly.
  Exporters may be added later, but they are adapters, not the core.
- The design creates a new schema/versioning responsibility for crunch.
- Provenance claims remain claims: crunch can verify binding and structure,
  not the real-world truth of a homepage, supplier, or license string.

## Alternatives Considered

**SPDX as the native format**: Rejected. It weakens crunch's model and makes
canonicalization harder.

**CycloneDX as the native format**: Rejected for the same reason.

**Sidecar report only, no first-class feature**: Rejected. It would miss the
opportunity to bind attestations to the build/store model and would remain easy
to ignore.

**Put provenance fields into the core derivation contract and derivation
hashes**: Rejected. Descriptive metadata changes should not force rebuilds by
default.
