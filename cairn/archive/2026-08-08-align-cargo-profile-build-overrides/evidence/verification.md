# Verification

Task-ID: V1
Covers: rust_package_planning.profile_build_override_defaults, rust_package_planning.profile_build_override_scope

## Question

Does Mantle apply Cargo's built-in build-override defaults to host work and record dual-use decisions?

## Inspected evidence

- Baseline task `12562` ran the existing build-script profile and host-dependency tests before implementation. All 3 tests passed.
- Task `12621` ran all focused `rust_plan::tests::` tests after implementation. The result was 208 passed and 0 failed.
- The same task ran all `cargo_profile::tests::` tests. The result was 8 passed and 0 failed.
- Task `12621` also ran direct rustfmt checks for the changed Rust files and `git diff --check`. Both passed.
- Task `12644` ran first-party Clippy for the Mantle binary with `-D warnings`. It passed.
- Task `12653` ran the completed task gate and full Cairn validation. Both passed.
- Task `12660` reran positive and negative receipt tests after the final fail-closed marker check. Both passed.
- Task `12665` reran the pure-core suite, dual-use checks, Clippy, rustfmt, and whitespace validation after final cleanup. All passed.
- Task `12670` reran the completed task gate, full Cairn validation, and whitespace validation. All passed.
- Task `12676` ran the sync dry run, sync execution, and post-sync Cairn validation. All passed.
- Task `12685` ran post-archive Cairn validation against the final archive. It passed.
- The exact output is in `evidence/post-archive-validation.txt`.
- Topology self-probe task `12629` wrote `/tmp/mantle-profile-build-override-self-probe/receipt.json`.
- The self-probe stopped at the existing `native-host-unit-graph-blocked` blocker with no unit executions.
- The earlier profile-codegen self-probe had the same blocker and no unit executions.
- Therefore, focused receipt tests record the intended changed values; the self-probe did not reach an affected unit.
- Pure-core tests cover `dev`, `test`, `release`, and `bench` defaults.
- Negative tests reject legacy release `OPT_LEVEL=3`, legacy dev `DEBUG=true`, and missing dual-use records.
- Native and Cargo-derived host tests check `opt-level=0`, `debuginfo=0`, and `codegen-units=256`.
- Host-dependency tests distinguish host-only and dual-use packages from the selected graph.
- Receipt tests record the build-override settings and the dual-use decision.
- A subprocess test proves ambient profile environment values cannot replace resolved values.

## Decision

Accepted. The pure profile core derives and validates the built-in build-override policy.

The rust-plan shell applies the policy to build scripts, proc macros, and host dependencies. It records dual-use decisions in derivation environment identity and unit receipts.

## Owner

Mantle agent.

## Next action

Use this resolver as the built-in base for root-manifest profile configuration.
