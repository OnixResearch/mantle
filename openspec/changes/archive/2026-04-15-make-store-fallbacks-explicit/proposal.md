# Make store fallbacks explicit

## Sequence

Step 3 of 6. This change depends on hermeticity-mode plumbing and applies it to
store-backed closure and `PathInfo` behavior.

## Why

Two store-layer fallbacks still weaken hermeticity claims:

- persistent `PathInfo` open failure quietly degrades to in-memory state,
- missing closure facts can warn and continue with a best-effort mount set.

Those are reasonable escape hatches for practical development, but they must be
explicit and strict mode must be able to reject them.

## What Changes

- classify persistent `PathInfo` fallback and degraded closure resolution as explicit audit events
- make strict mode fail before build execution on those conditions
- keep practical mode usable by surfacing the degraded conditions in the result
- add regression coverage for both strict and practical behavior

## Capabilities

### New Capabilities

- `strict-store-fallback-policy`: strict builds reject weakened store semantics
- `degraded-store-reporting`: practical builds report weakened store semantics explicitly

## Impact

- **Files**: `crates/crunch-store/src/handle.rs`, `crates/crunch-store/src/closure.rs`, and store-related integration tests
- **Behavior**: strict mode fails earlier on missing persistent state or missing closure facts
- **Testing**: add strict/practical coverage for both fallback paths
