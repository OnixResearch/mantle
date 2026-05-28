## Why

Native `rust-plan --execute-topology` now moves past host dependency producer coverage, but the first real `rustc` link can still fail before useful topology evidence because the hermetic child environment clears `PATH`. Rustup's target linker wrapper uses `#!/usr/bin/env bash`, so a valid absolute `rustc` invocation can fail with `env: 'bash': No such file or directory` even when the caller supplied a usable tool PATH.

## What Changes

- Preserve a bounded inherited `PATH` in Rust topology execution child processes after `env_clear()`.
- Apply the same tool PATH to build-script metadata execution so helper scripts can resolve their declared interpreters.
- Keep derivation-provided environment variables explicit and keep source/package planning independent of Cargo caches or network state.
- Add focused tests for PATH propagation and empty-PATH fail-closed behavior.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, archived evidence/tasks.
- **Validation**: focused Rust unit tests, self-probe, `cairn validate`, `git diff --check`.
