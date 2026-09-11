# Proposal: Stabilize SpaceWasm bundle evidence

## Why

ChaosControl cannot reproducibly admit the SpaceWasm bundle from pinned Mantle source. Mantle hashes raw Cargo and libtest logs into receipts and bundle members. Compilation order, test completion order, and elapsed times change those hashes across identical derivations.

A retained Nix check-mode rebuild rejected the upstream-test output as nondeterministic. Both recorded runs passed the same suite. The selected runner digest remained unchanged. See `evidence/baseline.md` for exact source identities, observations, and limitations.

Replacing the consumer's expected digest only admits one observed build. It does not repair the producer or establish reproducibility.

## What Changes

- Define a versioned, bounded contract for stable test facts and their canonical BLAKE3 identities. r[mantle.spacewasm_stable_evidence.contract]
- Separate stable bundle identity from run-specific diagnostics without losing diagnostic evidence or mislabeling transformed text as raw output. r[mantle.spacewasm_stable_evidence.diagnostics]
- Reject missing, duplicate, malformed, truncated, contradictory, unsupported, or failed evidence before successful publication. r[mantle.spacewasm_stable_evidence.denial]
- Prove reproducibility by rerunning report-producing dependencies and comparing complete bundles, not by reusing cached reports. r[mantle.spacewasm_stable_evidence.rebuild]
- Publish a reviewed migration contract and an immutable producer revision for independent ChaosControl admission. r[mantle.spacewasm_stable_evidence.handoff]

## Impact

- Immediate consumer: ChaosControl's `spacewasm-mvp-differential` acceptance check.
- Immediate outcome: identical admitted inputs and test facts produce identical stable bundle bytes through fresh report execution.
- Durable capability: a checked distinction between reproducible qualification facts and exact run observations.
- Maintenance owner: Mantle's SpaceWasm materialization owner, covering `crunch-spacewasm-core`, `crunch-spacewasm`, and `nix/spacewasm-reference.nix`.
- Consumer owner: ChaosControl retains profile admission, runtime execution, Campaign integration, and release decisions.
- Repeatability evidence: retained baseline failure, independent rebuild outputs, adversarial report fixtures, complete-member verification, and a frozen consumer check.
- Compatibility: the implementation must version changed report or bundle semantics. Existing receipts and their historical identities remain unchanged.

## Scope

The change covers the declared upstream unit suite, the spectest-address check, and every report-producing dependency of the SpaceWasm bundle. It also covers contract exports, member roles, diagnostic retention, verification, documentation, and producer publication evidence.

This package is a proposal only. All implementation and acceptance tasks remain open. Implementation must use a dedicated worktree from current `origin/main` and preserve unrelated Mantle work and active proofs.

## Out of Scope

- Updating ChaosControl's expected digest without a verified producer repair.
- Removing tests, changing selected features, or broadening the SpaceWasm support matrix.
- Mutating historical bundles, disabling identity checks, or adding consumer-side normalization.
- General Cargo log cleanup, a generic evidence framework, or unrelated bootstrap repairs.
- Proving SpaceWasm correctness, WebAssembly conformance, compiler correctness, sandbox effectiveness, Campaign adoption, or release eligibility.

## Success Criteria

- The stable bundle and all required members match across fresh executions of the exact admitted producer graph.
- Required test identities, outcomes, command arguments, feature selection, inputs, and artifact identities survive the projection.
- Negative controls deny every specified incomplete or contradictory result without a successful receipt.
- Exact raw diagnostics remain retrievable under the explicit retention contract, outside the stable identity cycle.
- ChaosControl independently verifies a published candidate without weakening its manifest or member checks.

## Affected Specs

- New delta: `spacewasm-stable-evidence`.
- Existing contract to preserve or explicitly version: `spacewasm-reference-materialization`, especially evidence, bundle completeness, and non-claims.
