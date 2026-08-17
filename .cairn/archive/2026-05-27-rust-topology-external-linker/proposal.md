## Why

After preserving `PATH`, native topology execution no longer fails on `/usr/bin/env bash`, but rustup's target `gcc-ld/ld.lld` wrapper can still reference a stale Nix store `ld-wrapper.sh`. The selected rustc invocation already uses an explicit external linker path from Cargo's unit args, so rustc's self-contained linker wrapper is unnecessary and can block topology evidence for host-specific reasons.

## What Changes

- Add a bounded runtime rustc argument that disables self-contained linker wrappers when the selected unit did not already choose a `link-self-contained` mode.
- Preserve any explicit unit-provided `link-self-contained` flag.
- Add focused tests for default injection and explicit-flag preservation.
- Re-run the self-probe to verify movement past the stale rustup linker-wrapper blocker.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, archived evidence/tasks.
- **Validation**: focused rustc-arg tests, self-probe, `cairn validate`, `git diff --check`.
