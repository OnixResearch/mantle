## Why

GCC 4.0 now has several bounded installed-`cc1` no-delegation proof inputs, but the central parity blocker remains the real native `cc1` source-build frontier. Continuing with more marker-only semantic inputs has diminishing returns unless we also make the native build frontier more inspectable.

This change adds a narrow, fail-closed frontier receipt around the current native `cc1` source-build attempt: it records the exact source/build markers, the current pass1 bridge fallback boundary, and the next actionable failure class without claiming full GCC correctness.

## What changes

- Add a checked `gcc.4.0` native-`cc1` build-frontier receipt that names the observed source/build boundary and retirement condition.
- Wire parity validation so `gcc.4.0` remains evidence-backed `partial` only when the receipt matches exact derivation markers and partial-only semantics.
- Add positive and negative regressions for matching evidence, marker drift, unsupported parity effect, and frontier overclaim.
- Preserve existing bounded `cc1` arithmetic/logical/local-vars, generator, demangle, placeholder, and boundary evidence.

## Non-goals

- Completing native GCC 4.0 `cc1`.
- Removing the pass1 bridge.
- Treating bounded installed-`cc1` marker outputs as general parser/codegen proof.
- Unblocking live-bootstrap, Guix, or StageX parity.
