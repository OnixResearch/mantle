## Why

The GCC 4.0 pass1 bridge now proves `_negdi2` and `_muldi3` semantic `libgcc.a` members. The next bounded low-risk member is `_lshrdi3`: a single unsigned 64-bit logical right shift helper used by older GCC code generation. Promoting it continues the correctness drain without claiming full native GCC or full libgcc completion.

## What Changes

- **Implement**: Promote `_lshrdi3` from a placeholder body to unsigned 64-bit logical-right-shift semantics in `bootstrap/gcc-4.0.ncl`.
- **Verify**: Rebuild the GCC 4.0 artifact and link a host semantic smoke against the produced `libgcc.a`.
- **Keep**: Preserve deterministic hand-written ar(5) archive generation and keep all other placeholder members out of scope.

## Capabilities

### Modified Capabilities
- `gcc40-libgcc-semantic-member`: Extend the one-member-at-a-time libgcc semantics promotion from `_negdi2` and `_muldi3` to `_lshrdi3`.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, OpenSpec bootstrap delta, task evidence.
- **APIs**: No public API change.
- **Dependencies**: No new dependency.
- **Testing**: `crunch eval`, generated shell syntax check, `crunch build bootstrap/gcc-4.0.ncl`, `ar`/`nm`, host-linked `_lshrdi3` smoke, `openspec validate --all --strict`.
