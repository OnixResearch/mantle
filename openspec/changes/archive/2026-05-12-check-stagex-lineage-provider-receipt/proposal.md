## Why

`seed-full.stagex-lineage` is the remaining StageX-specific parity blocker besides full self-build proof. Today the row is blocked only by prose: no derivation is expected and no machine-checkable lineage receipt is loaded by the parity report.

## What Changes

- Add a checked StageX lineage provider receipt contract for the `seed-full.stagex-lineage` parity row.
- Validate a repo-local scaffold receipt with closed provider kind `stagex-lineage`, explicit `scaffold-only` status, digest-shaped lineage fields, and no fallback events.
- Keep the row `partial`, not complete, until a real audited StageX lineage and self-build proof exist.

## Capabilities

### Modified Capabilities
- `bootstrap.stagex.selfbuild.proof`: parity must consume a checked lineage-provider receipt before the StageX lineage row can be evidence-backed partial.

## Impact

- **Files**: `src/bootstrap_parity.rs`, `bootstrap/evidence/stagex-lineage-provider-receipt.json`, OpenSpec bootstrap delta.
- **APIs**: No CLI schema change.
- **Dependencies**: None.
- **Testing**: Targeted parity tests, parity-report JSON, OpenSpec validation, whitespace check.
