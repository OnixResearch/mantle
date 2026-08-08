# Verification

Task-ID: V1
Covers: rust_package_planning.profile_root_manifest_authority, rust_package_planning.profile_custom_inheritance, rust_package_planning.profile_package_overrides, rust_package_planning.profile_selection

## Question

Does Mantle apply only the root Cargo profile table, resolve custom profiles and overrides, and record the effective policy?

## Inspected evidence

- Baseline task `12693` ran the existing profile-core and Cargo-import tests before implementation. The results were 8 profile tests and 7 import tests passed, with 0 failures.
- Task `12846` ran direct rustfmt checks for the changed Rust files, `git diff --check`, and first-party Clippy for the Mantle binary with `-D warnings`. All checks passed.
- Task `12847` ran the final focused suites. The results were 8 `cargo_profile` tests, 10 `cargo_profile_manifest` tests, 8 `cargo_import` tests, and 209 `rust_plan` tests passed, with 0 failures.
- Positive tests cover root-table snapshots, inherited custom profiles, named and wildcard overrides, build overrides, command defaults, Cargo import, rustc flags, metadata identity, build environments, and receipts.
- Negative tests reject missing inheritance, cycles, unknown keys, invalid values, parser limits, forbidden override settings, version-qualified package specs, unresolvable names, and parsed settings that Mantle cannot lower.
- Dependency manifest profile tables are ignored. Native package planning records `dependency-manifest-profile-settings-present-and-ignored` when such a table is present.
- Task `12854` ran a Cargo-oracle `rust-plan` over a temporary workspace with `[profile.fast]`, `inherits = "dev"`, `opt-level = 1`, and `codegen-units = 8`.
- The oracle receipt is `/tmp/mantle-profile-oracle-receipt-final.json`.
- The receipt selected profile `fast`, reported matched package, target-unit, and host-unit comparisons, and emitted `opt-level=1` and `codegen-units=8` in the planned rustc arguments.
- No new Cargo-oracle mismatch was found.

## Decision

Accepted. The pure profile core implements bounded parsing, inheritance, override precedence, and command selection.

The Rust planning shell snapshots the root table once, applies exact Cargo workspace membership for wildcard exclusions, and binds the effective settings to rustc arguments, metadata identity, build environments, and receipts. Cargo import uses the same resolver.

## Owner

Mantle agent.

## Next action

Run the Cairn task and repository validation gates, then sync and archive the change.
