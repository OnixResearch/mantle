# Align Cargo profile build overrides

## Why

Finding source: <https://doc.rust-lang.org/cargo/reference/profiles.html>,
section "Build Dependencies" (reviewed 2026-08-02).

Cargo's reference says all profiles, by default, do not optimize build
dependencies (build scripts, proc macros, and their dependencies). The
documented built-in build-override defaults are `opt-level = 0` and
`codegen-units = 256` for both `dev` and `release`, with debug info off when
possible.

Mantle's `build_script_profile_env` in `src/rust_plan.rs` diverges from this:
it reports `OPT_LEVEL=3` for release and bench build scripts, and `DEBUG=true`
for dev build scripts. Build scripts that branch on these values (cc-rs
optimization flags, `debug_assert!`-style build logic) see a different world
than they see under Cargo.

## What Changes

- Resolve build-script and proc-macro profile environment through Cargo's
  built-in build-override defaults: `OPT_LEVEL=0` and `DEBUG=false` for all
  profiles, with `NUM_JOBS=1` unchanged.
- Apply the same build-override defaults to the dependencies of build scripts
  and proc macros when those dependencies are compiled as host units.
- Keep a dual-use exception: when a package is both a build dependency and a
  normal target dependency built once, the target profile's `DEBUG` value
  applies, matching Cargo's "when possible" wording.
- Preserve the cleared-env boundary: values come from the pure resolver, not
  from ambient process environment.

## Impact

- **Planned files**: `src/rust_plan.rs` (`build_script_profile_env`,
  `append_build_script_profile_env`, host-unit env derivation, tests).
- **Testing**: positive tests for dev/release/bench build-override env,
  negative tests against the old release `OPT_LEVEL=3` and dev `DEBUG=true`
  behavior, and a dual-use package test.
- **Compatibility**: build-script receipts change because env values change.
  This is the intended parity fix.
