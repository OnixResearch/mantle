# I6 release operations (2026-10-01)

Scope: release-family CLI effect admission, bounded plans, capability ports, independent observations, and result classification. This record does **not** assert that the wider I6 inventory is closed or that the synthetic global-reproducibility fixture proves a real release.

## Before: detached `origin/main` CLI

A detached `origin/main` checkout at `da00f5842` produced the following with the same synthetic one-surface universe and policy, using `TMPDIR=/home/brittonr/scratch`:

- `crunch --json release global-reproducibility --universe … --policy … --report-path …`: JSON report `claim_class=blocked`, one missing-surface-evidence blocker, then the original JSON internal error, exit 3; the written report SHA-256 was `0f028ab6c28d5455ac055f4216a029bb516cdbc8ddf9408061a3438c23d65629`.
- The same command with `--evidence …`: JSON report `claim_class=eligible`, accepted witness `witness-a`, zero blockers, exit 0; the written report SHA-256 was `7cc590c8f565bbce5e24a8f7942bde7c628ab8fc09bd6fff041bc8a72d56b3a0`.

Both outputs are synthetic fixture behavior, not a global reproducibility proof.

An actual detached-baseline `--json release nix-witness` invocation against the deliberately missing `/home/brittonr/scratch/release-i6-missing-bundle` returned `{"error":"release bundle directory does not exist: /home/brittonr/scratch/release-i6-missing-bundle","code":3,"kind":"internal"}` and exit 3. It did not create the selected `/home/brittonr/scratch/release-i6-baseline-nix-receipt.json` file.

## Application and CLI boundary

`ReleaseEffectOperation` assigns distinct effect identities and declared authorities to concrete release actions. A `ReleaseEffectPlan` declares the destination and bounded measurement before a CLI port runs. The port retains the original CLI error; independent read-back of actual paths, directory shape, bytes, and counts supplies the classification before normal success output. Failed calls remain failed even when a selected path physically exists, without claiming that the invocation created that path or that the call was atomic. Nix `--require-match` still writes its mismatch receipt before failing. The gauntlet observes a written report before its terminal summary; rejecting that observation does not undo the file write. The former post-hoc constant `None` release classifier and its implementation-only tests are removed.

## Exercised contract checks

`TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/.cargo-target/release-i6-contract nix develop --offline --no-write-lock-file -c cargo test -p mantle-application-contract --lib release::tests:: --locked --offline`: **6 passed** after the shared lock integration. Includes destination/authority/bound contradictions and a failed write whose selected output remains observed without claiming success.

`TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/.cargo-target/release-i6-contract nix develop --offline --no-write-lock-file -c cargo clippy --no-deps -p mantle-application-contract --lib --locked --offline -- -D warnings`: **exit 0** after the project-contract owner's unrelated matcher warning was fixed. Only the four release-owned files `src/release_cmd.rs`, `src/function_address_binding_cmd.rs`, `src/verification_gauntlet_cmd.rs`, and `crates/mantle-application-contract/src/release.rs` were passed to `rustfmt --edition 2024`, **exit 0**.

## First post-cutover CLI smoke

The actual first post-cutover `/home/brittonr/scratch/rust-plan-integrated-target/current-tree-stable/debug/mantle` executable (SHA-256 `9bff7a785ac1e74bff7966cf95e8f1501daed8ce39095e16c407b14949409eb0`) was run with `TMPDIR=/home/brittonr/scratch`, the same synthetic universe/policy/evidence, and distinct after-cutover destinations. Without `--evidence` it wrote `release-i6-after-blocked.json`, reported one missing-surface-evidence blocker and `claim_class=blocked`, emitted the original JSON internal error and exit 3; the file SHA-256 exactly equaled the detached baseline `0f028ab6c28d5455ac055f4216a029bb516cdbc8ddf9408061a3438c23d65629`. With `--evidence` it wrote `release-i6-after-eligible.json`, reported `claim_class=eligible`, accepted `witness-a`, zero blockers, and exit 0; file SHA-256 exactly equaled the detached baseline `7cc590c8f565bbce5e24a8f7942bde7c628ab8fc09bd6fff041bc8a72d56b3a0`. Full captured output: `artifact://8727`. These synthetic fixtures do not establish a real-world global reproducibility claim.

The same first executable ran `--json release nix-witness` against the deliberately missing bundle with the detached-baseline flags: original JSON internal error `release bundle directory does not exist: /home/brittonr/scratch/release-i6-missing-bundle`, exit 3, and no selected `release-i6-after-nix-receipt.json` created.

## Focused gauntlet behavior

`TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/.cargo-target/release-baseline-i6 nix develop --offline --no-write-lock-file -c cargo test -p mantle --bin crunch verification_gauntlet_cmd::tests:: --locked --offline`: **3 passed**. The real strict-report fixture wrote an eligible file and then canonicalized that file to stdout while observing `None` selected output and its actual byte count. The rejected-observer case retained the actual report and its original error; this verifies a written report is not undone by failed classification. Six existing unrelated root-bin test-only `dead_code` warnings were emitted; none is in the touched release leaves.

## Complete release CLI regression suite

`TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/.cargo-target/release-baseline-i6 nix develop --offline --no-write-lock-file -c cargo test -p mantle --test release_cli --locked --offline`: **150 passed, 0 failed**. The real subprocess tests include matching Nix-artifact witness receipt, default bundle receipt destination, mismatch receipt that remains non-promoting, `--require-match` failure after writing its mismatch receipt, and deterministic proof source-drift rejection. Function-address binding, release bundle verification, reproducibility, attestation and witnessed rebuild CLI scenarios remained passing. Cargo emitted existing dead-code warnings from other root leaves; `nix` logged a busy evaluation cache SQLite warning but the command completed successfully.

## Strict root-bin lint boundary

`TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/.cargo-target/release-baseline-i6 nix develop --offline --no-write-lock-file -c cargo clippy --no-deps -p mantle --bin crunch --locked --offline -- -D warnings`: **exit 101**, 18 diagnostics (`artifact://9308`), none in the touched release leaves. Sixteen are `dead_code` in other root modules (`command_input`, `protected_exec`, `remote_nominal`, `source_built_fixed_point_resume`, `source_observation`); two are `clippy::err_expect` and `clippy::type_complexity` in the separately owned `rust_plan.rs`. This is not a passing strict root-bin gate. The isolated release contract strict Clippy above passed.
