# Release Provenance Specification

## Purpose

Admit deterministic release evidence only when it carries a validated content-bound genuine rebuild contract.

## Requirements

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
