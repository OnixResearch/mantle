## Why

Native Rust topology execution now advances through build-script metadata, but the next deterministic blocker is the `async-stream-impl` proc-macro host unit. Mantle invokes that proc-macro as a standalone rustc unit without the built-in `proc_macro` extern and without its normal host dependencies (`proc-macro2`, `quote`, `syn`), so rustc fails before downstream target units can consume the proc-macro artifact.

## What Changes

- Bind native proc-macro host units like Cargo host units: include the compiler-provided `proc_macro` extern and selected normal dependency artifacts in the host rustc invocation.
- Propagate selected package features into native rustc `--cfg feature=...` args so dependency crates expose the same feature-gated APIs Cargo selected.
- Keep dependency ordering explicit: proc-macro dependencies must be produced before the proc-macro host unit runs.
- Preserve the existing frontier evidence style: focused unit tests plus a clean HEAD self-probe that either advances beyond `async-stream-impl` or records the next deterministic blocker.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, archived change evidence.
- **Testing**: focused rust-plan unit tests, `cairn validate`, `git diff --check`, and clean self-probe receipt.
