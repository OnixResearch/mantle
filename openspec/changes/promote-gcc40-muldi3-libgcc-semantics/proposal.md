## Why

The GCC 4.0 pass1 bridge now proves one semantic `libgcc.a` member (`_negdi2`). The archived roadmap names `_muldi3` as the next low-risk arithmetic member, so continuing with one bounded promotion increases correctness without claiming full native GCC or full libgcc completion.

## What Changes

- **Implement**: Promote `_muldi3` from a placeholder body to signed 64-bit multiplication semantics in `bootstrap/gcc-4.0.ncl`.
- **Verify**: Rebuild the GCC 4.0 artifact and link a host semantic smoke against the produced `libgcc.a`.
- **Keep**: Preserve deterministic hand-written ar(5) archive generation and keep all other placeholder members out of scope.

## Capabilities

### Modified Capabilities
- `gcc40-libgcc-semantic-member`: Extend the one-member-at-a-time libgcc semantics promotion from `_negdi2` to `_muldi3`.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, OpenSpec bootstrap delta, task evidence.
- **APIs**: No public API change.
- **Dependencies**: No new dependency.
- **Testing**: `crunch eval`, generated shell syntax check, `crunch build bootstrap/gcc-4.0.ncl`, `ar`/`nm`, host-linked `_muldi3` smoke, `openspec validate --all --strict`.
