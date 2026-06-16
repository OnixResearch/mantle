# Rust source `rustc_apfloat` env frontier

Task-ID: rust-source-apfloat-env-frontier-2026-06-12
Covers: rust_package_planning.source_built_rust_seed_closure

## Oracle checkpoint

Question: What is the next honest implementation step after the real mrustc-to-Rust first-stage route reached the rustc crate graph?

Inspected evidence:

- Pueue task `96` preserved scratch `/tmp/mantle-rust-source-provider-V5aClY` and first-stage log `/tmp/mantle-rust-source-provider-V5aClY/mrustc-first-stage-build.log`.
- `rustc_apfloat` build script at `vendor/rustc_apfloat-0.2.3+llvm-462a31f5a5ab/build.rs` reads `CARGO_PKG_VERSION` and requires it to end with `+llvm-462a31f5a5ab`.
- The minicargo failure context printed intended package env `CARGO_PKG_VERSION=0.2.3+llvm-462a31f5a5ab`, but the build script observed `found \`0.1.0\`` before aborting.
- Existing evidence `rust-source-first-stage-real-route-progress-2026-06-12.md` records LLVM completion, `rustc_llvm` completion, and fail-closed stop at `rustc_apfloat` build-script version validation.

Decision: Do not promote the provider task. Treat the current frontier as first-stage process-environment isolation. Mantle should scrub inherited Cargo/build-script environment before invoking mrustc/minicargo/make so crate build scripts observe minicargo-owned package metadata, not Mantle's ambient package env.

Owner: `source-built-rust-seed-closure`.

Next action: Implement a bounded first-stage env scrub in `src/rust_source_provider.rs`, add a regression test that the generated script clears ambient `CARGO_PKG_*` / build-script vars before first-stage commands, then rerun the real-route probe and record either advancement past `rustc_apfloat` or the next deterministic fail-closed blocker.

## Success criteria

- The generated first-stage shell unsets a named allowlist/denylist of inherited Cargo and build-script variables before first-stage tool execution.
- Focused `rust_source_provider` tests prove the scrub is present and cannot silently regress.
- A real-route probe no longer stops because `rustc_apfloat` sees Mantle's `CARGO_PKG_VERSION=0.1.0`; if the probe still fails, the evidence records the exact remaining blocker.
- No final provider, provider smoke, one-shot self-build, or fixed-point proof is claimed until the real provider exists.
