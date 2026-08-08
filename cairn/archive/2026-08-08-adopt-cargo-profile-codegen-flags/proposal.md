# Adopt Cargo profile codegen flags

## Why

Finding source: <https://doc.rust-lang.org/cargo/reference/profiles.html>
(reviewed 2026-08-02).

Mantle's native rust-plan accepts `--profile` (default `dev`) but only uses the
profile name for build-script `PROFILE`, `OPT_LEVEL`, and `DEBUG` environment
variables. The rustc argument builders in `src/rust_plan.rs`
(`native_rustc_args`, Cargo-unit derivation args, integration-test args, and
dev-dependency lib args) emit no profile-derived codegen flags. A
`--profile release` topology therefore compiles every crate at the rustc
default optimization level, with no debuginfo policy, no debug-assertions
policy, and no overflow-checks policy.

Cargo's reference defines four built-in profiles (`dev`, `release`, `test`,
`bench`) with an explicit default table for `opt-level`, `debug`,
`debug-assertions`, `overflow-checks`, `lto`, `panic`, `incremental`,
`codegen-units`, `rpath`, `strip`, and `split-debuginfo`. Mantle must lower at
least the core codegen settings so profile selection changes real compiler
behavior and so profile identity separates build artifacts.

## What Changes

- Add a pure, deterministic Cargo-profile model with the built-in default
  table and `test` inherits `dev`, `bench` inherits `release`.
- Lower resolved profile settings into rustc arguments for every Rust unit:
  `-C opt-level`, `-C debuginfo`, `-C debug-assertions`,
  `-C overflow-checks`, and explicit `-C codegen-units`.
- Include the selected profile in the rustc metadata disambiguator so dev and
  release units of the same crate cannot collide.
- Record Mantle determinism deviations from Cargo defaults as explicit policy:
  incremental compilation stays disabled, and fixed `codegen-units` values are
  bound into receipts.
- Keep `lto`, `panic`, `rpath`, `strip`, and `split-debuginfo` at values that
  match rustc defaults for now, with the non-support recorded as bounded
  non-claims.

## Impact

- **Planned files**: `src/rust_plan.rs` (profile model, argument builders,
  metadata disambiguator, tests), receipts that bind profile settings.
- **Testing**: positive tests for dev/release/test/bench flag lowering and
  negative tests for unknown profiles, disambiguator collisions, and
  unsupported setting values.
- **Boundary**: pure profile resolution stays in functional-core functions.
  The argument builders remain the imperative shell.
- **Compatibility**: receipt and action-reference identities change because
  profile settings enter the rustc argument digest. This is intended.
