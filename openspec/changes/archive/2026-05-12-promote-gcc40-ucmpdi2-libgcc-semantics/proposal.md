## Why

The GCC 4.0 pass1 bridge now proves signed comparison via `_cmpdi2`. The matching remaining bounded comparison helper is `_ucmpdi2`: unsigned 64-bit comparison using GCC libgcc's 0/1/2 ordering contract. Promoting it completes the current comparison-pair drain without claiming full native GCC or full libgcc completion.

## What Changes

- **Implement**: Promote `_ucmpdi2` from a placeholder body to unsigned 64-bit comparison semantics in `bootstrap/gcc-4.0.ncl`.
- **Verify**: Rebuild the GCC 4.0 artifact and link a host semantic smoke against the produced `libgcc.a`.
- **Keep**: Preserve deterministic hand-written ar(5) archive generation and keep all other placeholder members out of scope.

## Capabilities

### Modified Capabilities
- `gcc40-libgcc-semantic-member`: Extend the one-member-at-a-time libgcc semantics promotion to `_ucmpdi2`.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, OpenSpec bootstrap delta, task evidence.
- **APIs**: No public API change.
- **Dependencies**: No new dependency.
- **Testing**: `crunch eval`, generated shell syntax check, `crunch build bootstrap/gcc-4.0.ncl`, `ar`/`nm`, host-linked `_ucmpdi2` smoke, `openspec validate --all --strict`.
