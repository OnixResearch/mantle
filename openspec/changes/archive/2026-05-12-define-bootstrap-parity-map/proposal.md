## Why

The current bootstrap specs contain many archived, per-stage live-bootstrap changes and some StageX-class proof rules, but they do not yet define one top-level parity map that tells future drain work when Crunch has reached parity with the complete live-bootstrap ladder, relevant Guix full-source bootstrap expectations, and the StageX no-quorum trust profile. Without that map, the cron drain can exhaust a narrow GCC/libgcc slice while still lacking an explicit whole-bootstrap completion target.

## What Changes

- Add a bootstrap parity matrix requirement that names the expected live-bootstrap, Guix, and StageX coverage axes.
- Require one canonical gap report that maps implemented derivations, placeholders, runtime-smoke status, proof bundles, and remaining trust roots.
- Require claim gating so Crunch cannot advertise parity until every mapped stage is implemented or explicitly justified as out-of-scope with replacement evidence.

## Capabilities

### Modified Capabilities
- `bootstrap`: Adds whole-chain parity criteria on top of existing per-stage bootstrap requirements and proof evidence.

## Impact

- **Files**: `openspec/specs/bootstrap/spec.md` via this delta.
- **APIs**: None directly; future implementation should likely add/extend bootstrap inventory validation commands.
- **Dependencies**: None.
- **Testing**: `openspec validate define-bootstrap-parity-map --strict` and `openspec validate --all --strict`.
