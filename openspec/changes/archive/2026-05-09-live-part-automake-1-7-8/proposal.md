## Why

The live-bootstrap drain is too broad: one blocked stage such as `bootstrap/automake-1.7.8.ncl` can stall grouped changes and hide the exact part that needs evidence. Upstream live-bootstrap documents `automake 1.7.8` as its own part in `parts.rst`; Crunch should track the matching derivation independently.

## What Changes

- Create an independent OpenSpec change for the `automake 1.7.8` part.
- Keep the scope bound to `bootstrap/automake-1.7.8.ncl` plus direct tests/evidence for that file.
- Let broad umbrella changes depend on this part instead of carrying its detailed debug state.

## Capabilities

### Modified Capabilities
- `bootstrap`: per-part live-bootstrap lineage and validation tracking.

## Impact

- **Files**: `bootstrap/automake-1.7.8.ncl`, part-specific evidence under this change.
- **APIs**: none expected unless this part exposes a new downstream bootstrap contract.
- **Dependencies**: upstream source pins and predecessor derivations for this part only.
- **Testing**: source-pin audit for `bootstrap/automake-1.7.8.ncl`, `crunch build bootstrap/automake-1.7.8.ncl`, part-specific smoke test, and host-leakage scan.

## Reference

- `~/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `automake 1.7.8`.
