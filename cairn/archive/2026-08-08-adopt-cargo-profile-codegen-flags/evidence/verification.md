# Verification

Task-ID: V1
Covers: rust_package_planning.profile_defaults_table, rust_package_planning.profile_codegen_flags, rust_package_planning.profile_unit_identity, rust_package_planning.profile_determinism_policy

## Question

Does Mantle apply built-in Cargo profile settings to each supported rustc path and bind the effective policy to unit evidence?

## Inspected evidence

- Baseline task `12408` ran from unchanged `origin/main`.
- The baseline metadata test passed: 1 passed, 0 failed.
- The baseline profile-environment tests passed: 2 passed, 0 failed.
- Task `12456` ran all focused `rust_plan::tests::` tests after implementation.
- The focused result was 205 passed, 0 failed.
- The same task ran `cargo fmt --check -p mantle -v`.
- That broad format command found an unrelated import-order difference in `src/source_built_fixed_point_shell.rs`.
- Baseline task `12486` found the same format difference on unchanged `origin/main`.
- Task `12463` checked `src/cargo_profile.rs` and `src/rust_plan.rs` directly with rustfmt. It passed.
- Task `12463` also ran `git diff --check`. It passed.
- Task `12497` ran first-party Clippy for the Mantle binary with `-D warnings`. It passed.
- Task `12494` ran the completed task gate and full Cairn validation. Both passed.
- Task `12513` ran the sync dry run, sync execution, and post-sync Cairn validation. All passed.
- Task `12530` ran post-archive Cairn validation against the final archive. It passed.
- The exact output is in `evidence/post-archive-validation.txt`.
- Baseline self-probe task `12461` wrote `/tmp/mantle-profile-codegen-origin-self-probe/receipt.json`.
- Post-change self-probe task `12458` wrote `/tmp/mantle-profile-codegen-self-probe/receipt.json`.
- Both self-probes stopped before unit execution with `native-host-unit-graph-blocked`.
- Both self-probes recorded 0 unit executions and 0 planned derivations.
- Therefore, the self-probes produced no artifact identities for comparison.
- Focused tests prove that dev and release metadata identities differ for identical crate facts.
- Focused tests also prove explicit codegen arguments for native, host, Cargo-derived, test, and dev-dependency paths.
- A subprocess test proves that ambient `RUSTFLAGS` cannot replace the resolved profile arguments.
- Receipt tests prove that `incremental = false` and the selected `codegen-units` value enter the unit receipt.

## Decision

Accepted. The pure profile core resolves all four built-in profiles and rejects unknown names and unsupported settings.

The rust-plan shell applies the resolved settings to each supported rustc path. Unit metadata and artifact identity include the selected profile.

The unchanged self-probe blocker prevents an end-to-end artifact comparison. This blocker is outside this change.

## Owner

Mantle agent.

## Next action

Use this profile core for built-in build overrides and root-manifest profile configuration.
