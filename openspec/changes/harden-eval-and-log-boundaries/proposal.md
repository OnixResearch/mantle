# Harden eval and log boundaries

## Why

The audit found two related quality problems at public boundaries.

First, user-reachable lazy-evaluation paths in `crates/crunch-eval` still rely
on `expect(...)` for shape and cache assumptions. Those assumptions may be
reasonable internally, but public callers should receive typed `crunch-eval`
errors instead of process panics when evaluated input or session state violates
those expectations.

Second, build-log persistence still uses best-effort writes in places where an
operator expects diagnostics to either exist or fail loudly. Silent log-write
failure makes it harder to trust saved-log paths and harder to distinguish a
real missing log from a write failure.

We need to harden those boundaries before more features build on them.

## What Changes

- make lazy session and root-extraction paths in `crunch-eval` return typed
  errors for user-reachable malformed-shape or stale-state cases instead of
  panicking
- surface build-log persistence failures explicitly in operator-facing output
  and keep saved-log references honest
- add regression coverage for malformed lazy-eval inputs and for diagnostic
  write-failure handling

## Capabilities

### New Capabilities

- `panic-free-lazy-eval-boundary`: lazy evaluation APIs fail with typed errors
  instead of panicking on user-reachable shape mismatches
- `explicit-log-persistence-failure`: log-write failures are surfaced instead of
  silently discarded

## Impact

- **Files**: `crates/crunch-eval/src/{lib,session}.rs`, `src/build_cmd.rs`,
  related diagnostics/reporting paths, and regression tests
- **APIs**: may refine error-return behavior on existing lazy-eval/public
  helper paths without changing their core purpose
- **Dependencies**: no new runtime dependencies expected
- **Testing**: needs targeted malformed-input tests and explicit log-write
  failure coverage

## Verification

A reviewer should expect this change to land with:

- targeted tests proving malformed or inconsistent lazy-eval shapes return
  `crunch-eval` errors and do not panic the process
- targeted tests proving build-log persistence failures are surfaced
  explicitly and do not emit fake saved-log paths
- broad `cargo test --workspace --lib --tests` still passing under the
  documented build environment

## Non-Goals

- remove every internal `debug_assert!` or impossible-state check in the repo
- redesign Nickel's own diagnostic model
- change successful build-log formatting when persistence succeeds
