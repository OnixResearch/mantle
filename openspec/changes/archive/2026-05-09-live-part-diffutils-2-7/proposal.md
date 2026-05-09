## Why

The live-bootstrap drain is too broad: one blocked stage such as `bootstrap/diffutils-2.7-musl.ncl` can stall grouped changes and hide the exact part that needs evidence. Upstream live-bootstrap documents `diffutils 2.7` as its own part in `parts.rst`; Crunch should track the matching derivation independently.

## What Changes

- Create an independent OpenSpec change for the `diffutils 2.7` part.
- Keep the scope bound to `bootstrap/diffutils-2.7-musl.ncl` plus direct tests/evidence for that file.
- Let broad umbrella changes depend on this part instead of carrying its detailed debug state.

## Capabilities

### Modified Capabilities
- `bootstrap`: per-part live-bootstrap lineage and validation tracking.

## Impact

- **Files**: `bootstrap/diffutils-2.7-musl.ncl`, part-specific evidence under this change.
- **APIs**: none expected unless this part exposes a new downstream bootstrap contract.
- **Dependencies**: upstream source pins and predecessor derivations for this part only.
- **Testing**: source-pin audit for `bootstrap/diffutils-2.7-musl.ncl`, `crunch build bootstrap/diffutils-2.7-musl.ncl`, part-specific smoke test, and host-leakage scan.

## Reference

- `~/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `diffutils 2.7`.
