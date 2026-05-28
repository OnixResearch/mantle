## Why

Native Rust topology now runs `aws-lc-sys` and `aws-lc-rs` build scripts successfully, but the next clean self-probe blocks while compiling `darling@0.20.11`. The target library consumes the `darling_macro` proc-macro host artifact, yet its direct rustc invocation lacks a bound `--extern darling_macro=...`, so rustc searches the sysroot and fails with `rustc_private`.

## What Changes

- Bind consumed proc-macro host artifacts into target rustc `--extern` surfaces even when Cargo's target-unit dependency artifact list omits a matching placeholder.
- Preserve existing placeholder rewrite behavior when a matching dependency artifact is present.
- Add produced proc-macro host artifact directories as deterministic target dependency search paths so downstream rlib metadata remains loadable.
- Add focused positive and negative tests for host-artifact extern insertion, no-duplicate rewriting, custom-build exclusion, and transitive search-path binding.
- Record whether the self-probe moves past the `darling_macro` sysroot blocker.

## Impact

- **Files**: `src/rust_plan.rs`, Cairn change/evidence files.
- **Testing**: focused `rust_plan::` tests, clean self-probe, `cairn validate --root .`, `git diff --check`.
