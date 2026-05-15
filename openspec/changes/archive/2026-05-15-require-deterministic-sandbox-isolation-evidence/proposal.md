## Why

Mantle now records deterministic proof sandbox profile identities and has regression tests for the fake sandbox runner, but deterministic-release eligibility still lacks a typed evidence object that binds those isolation regressions to the supported `mantle-proof-sandbox-v1:` profile family. A supported-looking profile string should not be enough to promote a release if the maintained isolation evidence is missing, stale, malformed, or for a different profile family.

## What Changes

- Add a deterministic sandbox isolation evidence receipt for supported proof sandbox profile families.
- Require release deterministic-claim eligibility to receive valid isolation evidence before treating `mantle-proof-sandbox-v1:` receipts as promotion evidence.
- Add positive and negative tests for missing, malformed, stale/failing, and profile-family-mismatched evidence.

## Capabilities

### Modified Capabilities
- `release-verification-tech`: deterministic-release promotion now requires typed maintained isolation evidence, not only matching output digests and supported sandbox profile identity strings.

## Impact

- **Files**: `crates/crunch-release-core/src/determinism.rs`, OpenSpec release verification tech delta.
- **APIs**: deterministic release eligibility gains an isolation-evidence input and keeps fail-closed behavior.
- **Testing**: targeted release-core determinism tests plus OpenSpec validation.
