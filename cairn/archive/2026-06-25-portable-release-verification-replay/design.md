# Design: portable release verification replay

## Context

Release evidence verification should be reviewable by an operator who receives a release bundle and declared proof sidecars. The current local proof used checkout-relative `target/` paths, which is acceptable for producing evidence but not sufficient for a portable replay claim.

## Decisions

### 1. Replay from a copied artifact set

The replay should copy the release evidence bundle and deterministic proof sidecar directory into a fresh scratch root. Verification should run using paths rooted in that scratch directory, not the original bundle path.

### 2. Keep provider proof bundled, deterministic proof explicit

Until deterministic proof artifacts become bundle-local, the portable set is the release bundle plus the deterministic proof receipt and sandbox isolation evidence. The provider fixed-point proof is already bundle-local and should be validated from the copied release bundle.

### 3. Include a fail-closed negative replay

The rail should deliberately omit or rename one required deterministic proof sidecar and assert `--require-deterministic-release` fails. This prevents the portable replay from proving only the happy path.

### 4. Preserve bounded wording

The replay evidence should state that it proves portable verification of the packaged-artifact deterministic claim and bundled provider fixed-point proof. It should not claim full bootstrap reproducibility, compiler correctness, full Cargo compatibility, or deployability.

## Validation

Validation is complete when the positive replay succeeds from a fresh scratch root, the negative replay fails closed, exact output is recorded in tracked Cairn evidence, and `cairn validate --root .` passes.
