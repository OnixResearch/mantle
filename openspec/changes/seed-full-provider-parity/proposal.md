## Why

`crunch bootstrap parity-report` currently models `seed-full` as one partial row shared by the Guix and StageX axes. That hides the actionable distinction between the Guix/source-root normalized provider contract, which can be checked from `bootstrap/seed-full.ncl`, and the StageX-class lineage provider proof, which must remain blocked until audited lineage evidence exists.

## What Changes

- **Split provider evidence semantics:** distinguish Guix source-root `seed-full` evidence from StageX lineage provider evidence in the parity map.
- **Validate the source-root provider contract:** let the parity report mark the Guix `seed-full` provider row complete only when the derivation carries the normalized provider metadata contract and does not retain legacy fetched-provider evidence.
- **Preserve fail-closed StageX claims:** keep a separate StageX seed-provider row blocked until lineage proof evidence exists.

## Capabilities

### Modified Capabilities
- `bootstrap.parity.gap-report`: reports `seed-full` provider progress with axis-appropriate blockers instead of one ambiguous shared row.
- `bootstrap.fullsource.provider.contract`: binds Guix provider parity to deterministic normalized-contract evidence.

## Impact

- **Files:** `src/bootstrap_parity.rs`, `tests/bootstrap_parity_cli.rs`, OpenSpec bootstrap spec updates.
- **APIs:** JSON row IDs may add a StageX-specific seed-provider row while preserving `seed-full` for Guix/source-root evidence.
- **Dependencies:** No new dependencies.
- **Testing:** Run bootstrap parity unit tests, CLI tests, JSON parity report, and strict OpenSpec validation.
