# Normalize build execution envelope

## Sequence

Step 2 of 6. This change builds on hermeticity-mode plumbing by defining the
canonical build envelope that strict mode relies on.

## Why

Host locale, timezone, user identity, shell identity, temp paths, and umask are
classic ambient impurity sources. Crunch already sets some sandbox defaults, but
not enough to support strong determinism claims.

## What Changes

- define one canonical execution envelope for sandboxed builds
- normalize reproducibility-sensitive env vars before builder start
- set an explicit build umask
- define strict-mode rules for derivation attempts to override canonical env
- add tests that host ambient state does not leak into the builder envelope

## Capabilities

### New Capabilities

- `canonical-build-envelope`: builds start from a stable execution envelope instead of host ambient state
- `strict-env-override-rules`: strict mode rejects unsafe overrides of reproducibility-sensitive vars

## Impact

- **Files**: `crates/crunch-build/src/build_request.rs`, sandbox launch code, and build-envelope tests
- **Behavior**: build env becomes more deterministic across hosts and sessions
- **Testing**: add targeted env and umask regression coverage

## Non-Goals

- change closure resolution policy
- change fetcher implementation
- add a full ambient-state matrix harness yet
