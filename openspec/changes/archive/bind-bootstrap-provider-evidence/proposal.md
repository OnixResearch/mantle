## Why

`crunch bootstrap parity-report` now identifies the highest-impact parity blockers as provider evidence rows: `seed-full` and `crunch.self-build`. Those rows gate Guix/source-root and StageX provider claims, but current artifacts do not make the selected provider kind a first-class, verifiable evidence field.

## What Changes

- **Seed provider metadata**: `bootstrap/seed-full.ncl` names the provider kind it represents (`source-root`) instead of relying on prose notes.
- **Self-build proof binding**: self-hosting proof manifests record the selected provider kind, and release evidence copies/verifies that provider kind in `proof_linkage`.
- **Fail-closed evidence**: proof/release validation rejects missing or unsupported provider kinds so parity claims cannot silently fall back to legacy evidence.

## Capabilities

### Modified Capabilities
- `bootstrap.fullsource.claim.evidence`: provider kind evidence becomes machine-readable.
- `bootstrap.stagex.selfbuild.proof`: self-build proof linkage binds selected provider kind.
- `release-evidence`: release manifests carry and verify provider kind linkage.

## Impact

- **Files**: `bootstrap/seed-full.ncl`, `scripts/prove-self-hosting.sh`, `tests/self_hosting.rs`, `crates/crunch-release-core/src/manifest.rs`, `src/release_evidence.rs`, release CLI tests.
- **Testing**: targeted release evidence tests, self-hosting manifest tests, parity CLI/report tests, OpenSpec validation.
