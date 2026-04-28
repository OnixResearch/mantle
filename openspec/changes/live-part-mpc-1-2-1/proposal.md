## Why

The live-bootstrap drain is too broad: one blocked stage such as `bootstrap/mpc-1.2.1.ncl` can stall grouped changes and hide the exact part that needs evidence. Upstream live-bootstrap's checked-in `steps/manifest` and `steps/mpc-1.2.1/` implementation use `mpc-1.2.1`; its current `parts.rst` heading says `mpc 3.2.1`, which this change records as an upstream documentation mismatch instead of changing Crunch's part identity.

## What Changes

- Create an independent OpenSpec change for the `mpc 1.2.1` part.
- Keep the scope bound to `bootstrap/mpc-1.2.1.ncl` plus direct tests/evidence for that file.
- Let broad umbrella changes depend on this part instead of carrying its detailed debug state.

## Capabilities

### Modified Capabilities
- `bootstrap`: per-part live-bootstrap lineage and validation tracking.

## Impact

- **Files**: `bootstrap/mpc-1.2.1.ncl`, part-specific evidence under this change.
- **APIs**: none expected unless this part exposes a new downstream bootstrap contract.
- **Dependencies**: upstream source pins and predecessor derivations for this part only.
- **Testing**: source-pin audit for `bootstrap/mpc-1.2.1.ncl`, `crunch build bootstrap/mpc-1.2.1.ncl`, part-specific smoke test, and host-leakage scan.

## Reference

- `~/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, heading `mpc 3.2.1` (upstream documentation mismatch).
- `~/git/pi-repos/fosslinux--live-bootstrap/steps/manifest`, entry `build: mpc-1.2.1`.
- `~/git/pi-repos/fosslinux--live-bootstrap/steps/mpc-1.2.1/`.
