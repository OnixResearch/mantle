# Adopt Cargo profile manifest and selection model

## Why

Finding source: <https://doc.rust-lang.org/cargo/reference/profiles.html>,
sections "Profile settings", "Profile selection", "Custom profiles", and
"Overrides" (reviewed 2026-08-02).

Mantle currently accepts only a `--profile` string with built-in names. Real
workspaces configure profiles in the root `Cargo.toml` `[profile]` table:
custom profiles such as `release-lto` with an `inherits` key, per-package
overrides such as `[profile.dev.package.foo]`, and `"*"` wildcard overrides.
Cargo also defines a fixed profile-selection table per command and a strict
override precedence.

The companion changes `adopt-cargo-profile-codegen-flags` and
`align-cargo-profile-build-overrides` define the built-in setting table and
the built-in build-override defaults. This change adds the manifest-driven
configuration layer on top of them.

## What Changes

- Treat only the workspace root manifest `[profile]` table as authoritative.
  Ignore profile settings in dependency manifests and record a non-claim when
  they appear.
- Support custom profiles with a required `inherits` key. Resolve settings in
  order: custom setting, inherited base setting, built-in default. Reject
  inheritance cycles and unknown setting keys with deterministic blockers.
- Support `[profile.<name>.package.<name>]` overrides, the `"*"` wildcard for
  non-workspace members, and the `build-override` table, with Cargo's
  first-match precedence: named package, `"*"`, `build-override`, profile,
  built-in default.
- Reject `panic`, `lto`, and `rpath` inside overrides, as Cargo does.
- Adopt Cargo's profile-selection rules: `--release` equals
  `--profile release`, build-like commands default to `dev`, test defaults to
  `test`, bench defaults to `bench`.
- Extend the `cargo import` scaffold to resolve supported custom profiles
  instead of blocking every non-debug/release name.

## Impact

- **Planned files**: `src/rust_plan.rs` manifest parsing and profile
  resolution, `src/cargo_import.rs` profile blocker, focused tests.
- **Testing**: positive tests for custom profile resolution, override
  precedence, and selection rules. Negative tests for missing `inherits`,
  inheritance cycles, unknown keys, forbidden override settings, dependency
  manifest profiles, and version-qualified package specs.
- **Dependency**: builds on the built-in table from
  `adopt-cargo-profile-codegen-flags`.
