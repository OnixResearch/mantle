## Why

Mantle now has strong release-specific evidence: provider-bound proof material, deterministic release receipts, and an Aspen external witness agreement for one packaged release. That is not the same as global reproducibility. A global claim would say every admitted Mantle build surface in a declared universe can be replayed from declared inputs with matching outputs across the required witness matrix.

Without a Cairn requirement, release notes, readiness checks, and operator summaries can accidentally promote a bounded release result into a broader claim. This change makes the stronger claim explicit, evidence-gated, and fail-closed.

## What Changes

- Add verification-evidence requirements for global reproducibility claim admission.
- Define a durable `mantle-global-reproducibility-report-v1` evidence shape and claim classes.
- Require the build universe, policy, witness matrix, receipts, source/toolchain provenance, and blockers to be digest-bound before any global claim is allowed.
- Keep existing release-specific independent witness agreement wording bounded unless the new global gate passes.

## Impact

- **Files**: `cairn/changes/global-reproducibility-claims/*`, `cairn/specs/verification-evidence/spec.md` after sync/archive, future implementation in release/evidence reporting code.
- **Testing**: future positive fixture for a small fully admitted universe; future negative fixtures for missing evidence, digest mismatch, unsupported surface, weak hermeticity, and incomplete witness quorum.
