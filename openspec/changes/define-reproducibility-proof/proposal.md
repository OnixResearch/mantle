## Why

Mantle has several proof artifacts (self-build proof bundles, BLAKE3 final proof hashes, release evidence bundles, and witness rebuilds), but the project still needs a crisp operator-facing contract for when it is valid to say an artifact is reproducible. Without that contract, future proof work can accidentally conflate bundle-local integrity, self-proof validity, single-machine rebuild agreement, independent witness agreement, and full-source bootstrap claims.

## What Changes

- Define a canonical reproducibility proof protocol for release artifacts.
- Require proof claims to name their class, environment assumptions, inputs, recipe, clean rebuild stores, output digest set, and comparison verdict.
- Require BLAKE3 for Mantle-owned artifact comparisons, with explicit reasons for interoperability digests.
- Specify fail-closed behavior for mismatches, dirty inputs, unsupported recipes, and incomplete evidence.
- Provide implementation tasks for CLI/reporting, tests, docs, and release-verification integration.

## Capabilities

### Modified Capabilities
- `release.verification.tech.reproducibility.proof`: adds the technical contract for proving release-artifact reproducibility.
- `release.verification.tech.witness.rebuild.cli`: clarifies that witness rebuild evidence is one accepted reproducibility proof input, not an ad hoc recipe.

## Impact

- **Files**: `openspec/specs/release-verification-tech/spec.md`, likely release proof/witness report code and docs in implementation.
- **APIs**: may add a `mantle release reproducibility-report` or equivalent verifier/report output.
- **Dependencies**: no new mandatory runtime dependency; proof commands may use existing build sandbox and BLAKE3 tooling.
- **Testing**: canonical report serialization tests, positive/negative verification cases, mismatch tests, docs/examples checks, OpenSpec validation.
