## Why

The GCC 4.0 pass1 bridge now proves `_negdi2`, `_muldi3`, and `_lshrdi3` semantic `libgcc.a` members. The next bounded low-risk member is `_ashldi3`: a signed 64-bit arithmetic-left-shift helper whose result bits match left shift for representative non-overflowing counts. Promoting it continues the correctness drain without claiming full native GCC or full libgcc completion.

## What Changes

- **Implement**: Promote `_ashldi3` from a placeholder body to 64-bit left-shift semantics in `bootstrap/gcc-4.0.ncl`.
- **Verify**: Rebuild the GCC 4.0 artifact and link a host semantic smoke against the produced `libgcc.a`.
- **Keep**: Preserve deterministic hand-written ar(5) archive generation and keep all other placeholder members out of scope.

## Capabilities

### Modified Capabilities
- `gcc40-libgcc-semantic-member`: Extend the one-member-at-a-time libgcc semantics promotion to `_ashldi3`.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, OpenSpec bootstrap delta, task evidence.
- **APIs**: No public API change.
- **Dependencies**: No new dependency.
- **Testing**: `crunch eval`, generated shell syntax check, `crunch build bootstrap/gcc-4.0.ncl`, `ar`/`nm`, host-linked `_ashldi3` smoke, `openspec validate --all --strict`.
