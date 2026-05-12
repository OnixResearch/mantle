## Why

`crunch.self-build` remains a Guix/StageX parity blocker with only prose saying release evidence must bind the selected provider kind. The release evidence path already validates provider-kind linkage, but the bootstrap parity map has no checked evidence contract for that row.

## What Changes

- Add a checked self-build provider-kind linkage receipt contract for `crunch.self-build`.
- Require the receipt to bind `proof_identity.selected_provider_kind`, `proof_linkage.selected_provider_kind`, and `prerequisites.provider_kind` to the same closed provider kind.
- Keep `crunch.self-build` `expected_complete=false`, so a valid receipt can only justify evidence-backed `partial` until full self-build proof exists.

## Capabilities

### Modified Capabilities
- `bootstrap.stagex.selfbuild.proof`: provider-kind linkage becomes machine-checked by the parity report.

## Impact

- **Files**: `src/bootstrap_parity.rs`, OpenSpec bootstrap delta.
- **APIs**: No CLI schema change; parity notes become more actionable.
- **Dependencies**: None.
- **Testing**: targeted parity unit tests, parity-report JSON, OpenSpec validation, whitespace check.
