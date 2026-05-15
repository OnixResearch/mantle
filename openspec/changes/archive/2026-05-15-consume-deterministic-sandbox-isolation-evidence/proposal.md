## Why

Release reproduce now emits `deterministic-build-proof.json` and `deterministic-sandbox-isolation-evidence.json`, and release-core can decide deterministic-release eligibility from those artifacts. The user-facing `mantle release verify` path still only verifies the bundle/reproducibility report, so deterministic promotion is not yet tied to operator-visible verification.

## What Changes

- **Verifier input**: `mantle release verify` accepts deterministic proof and sandbox isolation evidence artifact paths.
- **Fail-closed option**: `--require-deterministic-release` makes verification fail unless the artifacts prove deterministic-release eligibility.
- **Reporting**: JSON and text verification output report deterministic status, artifact paths/digests, and blockers.

## Capabilities

### Modified Capabilities
- `release-verification-tech`: Release verification consumes generated deterministic proof artifacts instead of leaving them as reproduce-only outputs.

## Impact

- **Files**: `src/main.rs`, `src/release_cmd.rs`, release CLI tests, release-verification OpenSpec.
- **APIs**: CLI-only additions; release-core API remains unchanged.
- **Testing**: CLI tests for valid deterministic promotion, missing evidence failure, and malformed/stale evidence failure.
