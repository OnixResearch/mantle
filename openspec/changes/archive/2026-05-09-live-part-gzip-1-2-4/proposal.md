## Why

The live-bootstrap drain is too broad: one blocked stage such as `bootstrap/gzip-tcc.ncl` can stall grouped changes and hide the exact part that needs evidence. Upstream live-bootstrap currently labels the `parts.rst` section as `gzip 1.2.5`, but the matching implemented step directory and source pin are `steps/gzip-1.2.4` / `gzip-1.2.4.tar.gz`. Crunch should track the implemented derivation and record the heading mismatch explicitly.

## What Changes

- Create an independent OpenSpec change for the implemented `gzip 1.2.4` part and document the upstream heading mismatch.
- Keep the scope bound to `bootstrap/gzip-tcc.ncl` plus direct tests/evidence for that file.
- Let broad umbrella changes depend on this part instead of carrying its detailed debug state.

## Capabilities

### Modified Capabilities
- `bootstrap`: per-part live-bootstrap lineage and validation tracking.

## Impact

- **Files**: `bootstrap/gzip-tcc.ncl`, part-specific evidence under this change.
- **APIs**: none expected unless this part exposes a new downstream bootstrap contract.
- **Dependencies**: upstream source pins and predecessor derivations for this part only.
- **Testing**: source-pin audit for `bootstrap/gzip-tcc.ncl`, `crunch build bootstrap/gzip-tcc.ncl`, part-specific smoke test, and host-leakage scan.

## Reference

- `~/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `gzip 1.2.4`.
