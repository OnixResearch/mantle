# Tighten self-build proof hermeticity

## Sequence

Step 5 of 6. This change applies strict hermeticity policy to later self-build
and proof stages after bootstrap-tool roots exist.

## Why

Stage0 still has legitimate first-bootstrap host prerequisites. Later proof
stages are different: once crunch-built `busybox` and `bwrap` roots exist,
stage1 and later should bind to those exact outputs and stop tolerating host
fallback rediscovery.

## What Changes

- thread strict hermetic mode into self-build and the checked-in proof helper
- reject host fallback sandbox-tool discovery in later proof stages once crunch-built tool roots exist
- report exact bootstrap-tool provenance and fallback events in proof output
- add regression coverage for later-stage fallback rejection

## Capabilities

### New Capabilities

- `strict-self-build-proof-mode`: later proof stages can make a stronger no-host-fallback claim
- `proof-tool-provenance-reporting`: proof output records exact bootstrap-tool identity and fallback use

## Impact

- **Files**: `src/self_build.rs`, `scripts/prove-self-hosting.sh`, proof tests, and proof-summary rendering
- **Behavior**: strict proof runs fail if stage1+ falls back to host tool discovery
- **Testing**: add stage1/stage2 regression coverage for fallback rejection
