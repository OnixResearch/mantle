## Why

GCC 4.0 now has a checked native `cc1` build-frontier receipt, but the receipt records a static frontier: the native source-build attempt reaches a TinyCC/Mes `c-parse`/`gengtype-yacc.c` boundary and then installs the pass1 bridge. That is useful evidence, but the next product value is reducing the frontier or proving that a narrower source-build probe reaches a later, more actionable boundary.

This change plans one bounded native-source probe around the current `gcc.4.0` `cc1` build frontier. It must produce inspectable evidence of either a small forward movement or a more precise stable blocker, without treating the probe as native compiler correctness.

## What changes

- Add one narrow native `cc1` source-frontier probe or receipt update tied to `bootstrap/gcc-4.0.ncl`.
- Record exact attempted command/patch scope, observed frontier marker, and whether the frontier moved beyond the current `gengtype-yacc.c`/TinyCC-Mes boundary.
- Wire validation so stale frontier evidence, missing markers, or full-parity overclaims fail closed.
- Preserve existing bounded installed-`cc1` arithmetic/logical/local-vars evidence and keep `gcc.4.0` `partial`.

## Non-goals

- Completing native GCC 4.0 `cc1`.
- Removing the pass1 bridge unless the native source build actually succeeds far enough to replace it with evidence.
- Adding another installed-`cc1` marker-only semantic slice.
- Unblocking live-bootstrap, Guix, or StageX parity.
