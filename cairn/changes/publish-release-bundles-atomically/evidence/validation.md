# Atomic release publication validation evidence

Date: 2026-07-12
Change: `publish-release-bundles-atomically`
Implementation commit: `6d860f8c`

## Scope and claim

This evidence covers the pure publication plan/state machine, capability-scoped sibling staging, manifest-last production verification, Linux atomic no-replace commit, failure isolation, exact ownership-marker stale-stage quarantine, fresh retry, and the named positive/negative/race/failure-injection matrix.

The supported claim is local atomic visibility and no-clobber publication of a production-verified bundle. It does not claim filesystem crash durability, power-loss persistence, artifact correctness, release eligibility, reproducibility, or verifier soundness.

All Cargo commands used isolated scratch roots under `/tmp/mantle-agent-atomic-*-target` and `/tmp/mantle-agent-atomic-*-tmp`; the repository `target` path was not used.

## Baseline before core changes

- `cargo test -p crunch-release-core`
  - Result: `179 passed; 0 failed` and `0` doctests.
- `cargo test -p mantle --bin mantle release_evidence::`
  - Result: `26 passed; 0 failed; 1346 filtered out`.

The first pueue baseline attempt did not start because pueue's environment lacked `cargo`. It produced no test result. The baseline was rerun with the checked nightly and documented clang/mold/OpenSSL environment before implementation edits.

## Focused final validation

- `rustfmt --check --edition 2024 crates/crunch-release-core/src/publication.rs crates/crunch-release-core/src/lib.rs src/release_capability.rs src/release_evidence.rs src/release_publication.rs src/release_tree_copy.rs src/main.rs tests/release_cli.rs tools/tracey_refs.rs`
  - Result: PASS.
- `cargo test -p crunch-release-core`
  - Result: `188 passed; 0 failed` and `0` doctests.
- `cargo check -p crunch-release-core --target wasm32-unknown-unknown`
  - Result: PASS.
- `cargo clippy -p crunch-release-core --all-targets -- -D warnings`
  - Result: PASS.
- `cargo test -p mantle --bin mantle release_evidence::`
  - Result: `35 passed; 0 failed; 1346 filtered out`.
- `cargo test -p mantle --test release_cli release_create_`
  - Result: `13 passed; 0 failed; 115 filtered out`.

The root release test lane includes these named publication regressions:

- `publication_production_path_is_manifest_last_and_atomically_visible`
- `named_phase_failpoints_leave_final_absent_and_fresh_retry_succeeds`
- `staged_verification_failure_never_publishes_tampered_artifacts`
- `source_identity_drift_after_planning_fails_before_manifest_publication`
- `preexisting_files_and_directories_are_unchanged_and_block_staging`
- `preexisting_destination_symlink_is_unchanged_and_never_followed`
- `concurrent_file_and_directory_winners_are_never_clobbered`
- `concurrent_symlink_winner_is_never_replaced_or_followed`
- `cleanup_failure_is_retried_via_owned_stage_quarantine_only`

The pure core lane separately covers deterministic BLAKE3 identity, identity sensitivity to release/layout/input/policy facts, exclusion of random stage state, existing-destination blockers, invalid layout/policy blockers, staged-artifact mismatch, valid transitions, skipped/terminal transition rejection, verification rejection, commit race, and bounded cleanup-failure augmentation.

## Cairn lifecycle validation

The Mantle flake does not export a `cairn` app, so `nix run path:$PWD#cairn -- ...` failed before lifecycle validation. The canonical local Cairn flake was then used:

- `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`
  - Result: `valid: true`, `issues: []`, `change_issues: []`, `spec_issues: []`, `35` specs and `8` changes.
- `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal publish-release-bundles-atomically --root .`
  - Result: `PASS`, receipt `3ff00c943c92efb1d97b11ffb58ce01c9db80c6a08acf24eb9a4caac6189b627`.
- `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design publish-release-bundles-atomically --root .`
  - Result: `PASS`, receipt `dde19fb85b7b471bfca0283993ada36c6062ca27a5a3d4d79ad4d8e7c9f17e89`.
- `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks publish-release-bundles-atomically --root .`
  - Result before task checkbox/evidence update: `PASS`, receipt `c976604d5887fea8196301d1301e41b7c21a4c5f125a05c2b94b7c945dd9db56`.

After this evidence file and all completed task markers were written, lifecycle validation was rerun:

- `validate`: `valid: true`, no issues, `35` specs and `8` changes.
- proposal gate: `PASS`, receipt `ec74dad56049b0e4d9ddc3c3c527a83489819f68889d39a55f1bed0afce53a76`.
- design gate: `PASS`, receipt `96701030539d1a46bbd8f2fbb26c3a2dcf9ef30c60487894b916612cd0d2527f`.
- tasks gate: `PASS`, receipt `e62590325aafd7ebd8829fe2ec5c40a880935879e9144cf405b280aabe502831`.

No Cairn sync or archive command was run.

## Broader pre-existing validation blockers

These are not changed by implementation commit `6d860f8c` and do not block the focused publication evidence:

- `cargo fmt --check -p mantle -p crunch-release-core` reaches unrelated pre-existing formatting diffs, including `tests/trust_policy_offline_rail.rs:300`. The exact touched-file rustfmt check above passes.
- `cargo clippy -p mantle --lib --no-deps -- -D warnings` is blocked in unchanged `src/build_correctness.rs:495` (`result_large_err`) and `src/build_correctness.rs:543` (`collapsible_if`). The no-std publication core clippy gate passes with warnings denied.

## Integration hardening

The atomic-publication ADR was renumbered from 0020 to 0021 during integration because ADR 0020 already records the Trellis acceptance boundary.

Adversarial integration review identified a post-verification replacement window in the test adapter: a stage artifact could change during the `PreCommit` callback after the first production verification and then be renamed publicly. The shell now reruns production bundle verification after `PreCommit` and before removing the ownership marker or attempting the no-replace rename. A negative fixture mutates the staged binary in that window and proves the final path remains absent.

Pueue task `697` ran the focused pre-commit replacement test plus all 13 release-create CLI tests. Pueue task `710` then ran the full root release-evidence lane (36 passed, 0 failed), the complete release core (194 passed, 0 failed), and strict core Clippy with warnings denied.

Pueue task `713` ran repository validation and proposal, design, and tasks gates with the generated Mantle policy. Validation reported 9 active changes, 36 specs, no issues, and `valid: true`; all three gates returned no issues, `valid: true`, and `verdict: PASS` under policy hash `d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c`.

## Accepted-spec synchronization

Pueue task `718` dry-ran and executed Cairn sync with no blockers. The execute receipt reported identical accepted-spec before/after hashes (`3ad907ef326c43be2888717905fccb5ad99fbb77bbd0e4ad4afb5661d0f643f6`), and the atomic-publication requirements were absent, so the reviewed delta was appended manually.

The first standalone comparison task (`733`) failed before evidence because the ambient Cargo wrapper could not launch; it produced no validation result. Pueue task `737` reran in the documented development environment with wrappers cleared and proved the accepted requirement suffix is byte-for-byte equal to the reviewed delta: 4624 bytes, BLAKE3 `33720bb3ae7ea6486f1ebe4174a63150189892568155aaa0855c051c06605cdd`. Post-sync validation remained `valid: true` with 9 active changes and 36 specs.
