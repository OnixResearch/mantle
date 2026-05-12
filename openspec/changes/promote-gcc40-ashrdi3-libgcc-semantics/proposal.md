## Why

The GCC 4.0 pass1 bridge now proves four semantic `libgcc.a` members (`_negdi2`, `_muldi3`, `_lshrdi3`, `_ashldi3`). The next bounded member is `_ashrdi3`: signed 64-bit arithmetic right shift. Promoting it completes the paired shift-helper drain for right/left shifts without claiming full native GCC or full libgcc completion.

## What Changes

- **Implement**: Promote `_ashrdi3` from a placeholder body to signed 64-bit arithmetic-right-shift semantics in `bootstrap/gcc-4.0.ncl`.
- **Verify**: Rebuild the GCC 4.0 artifact and link a host semantic smoke against the produced `libgcc.a`.
- **Keep**: Preserve deterministic hand-written ar(5) archive generation and keep all other placeholder members out of scope.

## Capabilities

### Modified Capabilities
- `gcc40-libgcc-semantic-member`: Extend the one-member-at-a-time libgcc semantics promotion to `_ashrdi3`.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, OpenSpec bootstrap delta, task evidence.
- **APIs**: No public API change.
- **Dependencies**: No new dependency.
- **Testing**: `crunch eval`, generated shell syntax check, `crunch build bootstrap/gcc-4.0.ncl`, `ar`/`nm`, host-linked `_ashrdi3` smoke, `openspec validate --all --strict`.
