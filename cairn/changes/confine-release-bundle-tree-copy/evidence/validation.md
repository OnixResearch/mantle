# Validation Evidence

Question: Does Mantle reject hostile release-tree shapes before bundle mutation and keep every tested write within the declared destination root?

Inspected evidence: the pre-change exploit baseline, pure planner tests, no-follow capability-shell tests, release-evidence production tests, release CLI integration tests, no-std target check, focused formatting, and focused lint output listed below.

Decision: Implementation evidence is sufficient for the completed change tasks. The claim is bounded to untrusted release-tree shape, planned entry kinds/paths/targets, and the tested source/destination type-drift boundary; copied artifact semantics remain out of scope.

Owner: Mantle release evidence implementation.

Next action: keep the change active for serial integration; do not sync or archive it in this worktree.

## Pre-change exploit baseline

Command:

```text
cargo test -p mantle --bin mantle release_evidence::tests::create_release_bundle_rejects_directory_symlink_escape_without_external_writes -- --exact --nocapture
```

Result before the core edit:

```text
assertion `left == right` failed
left:  attacker-replacement
right: external-sentinel
test result: FAILED. 0 passed; 1 failed
```

This established that the production release bundle path followed a source directory symlink, recreated it at the destination, and overwrote an external sentinel.

## Current focused checks

```text
rustfmt --check --edition 2024 --config skip_children=true \
  crates/crunch-release-core/src/lib.rs \
  crates/crunch-release-core/src/tree_copy.rs \
  src/main.rs src/release_capability.rs src/release_evidence.rs \
  src/release_tree_copy.rs tests/release_cli.rs tools/tracey_refs.rs
PASS

cargo test -p crunch-release-core --lib
PASS: 150 passed; 0 failed

cargo check -p crunch-release-core --target wasm32-unknown-unknown
PASS

cargo test -p mantle --bin mantle release_tree_copy::tests -- --nocapture
PASS: 6 passed; 0 failed

cargo test -p mantle --bin mantle release_capability::tests -- --nocapture
PASS: 6 passed; 0 failed

cargo test -p mantle --bin mantle release_evidence::tests -- --nocapture
PASS: 26 passed; 0 failed

cargo test -p mantle --test release_cli release_create_ -- --nocapture
PASS: 12 passed; 0 failed

cargo clippy -p crunch-release-core --lib -- -D warnings
PASS

git diff --check
PASS
```

The production external-sentinel checks assert both byte preservation and absence of attacker-created external paths. Positive fixtures assert that nested files and supported internal file/directory symlinks copy successfully, source/destination tree digests match, and changing only symlink target text changes the BLAKE3 tree digest.

## Broader pre-existing blockers

The package-wide `cargo fmt --check -p mantle -p crunch-release-core` remains blocked by formatting drift in unrelated pre-existing files such as `tests/trust_policy_offline_rail.rs`; the touched-file rustfmt check above passes.

Strict root-package Clippy remains blocked before the changed shell code by two unrelated existing diagnostics in `src/build_correctness.rs` (`result_large_err` and `collapsible_if`). `cargo clippy -p crunch-release-core --lib -- -D warnings` passes, and non-denying root Clippy reported no diagnostics in `src/release_tree_copy.rs`.

## Cairn gates

Initial gate run before checking the final gate task:

```text
cairn validate --root .
PASS: valid=true; changes=18; specs_validated=43; issues=[]

cairn gate proposal confine-release-bundle-tree-copy --root .
PASS: verdict=PASS; receipt_hash=65ed3d466bd34332159f135fd2240b892bc3374d2ca5c4c0d9df954f18ba464a

cairn gate design confine-release-bundle-tree-copy --root .
PASS: verdict=PASS; receipt_hash=792643c21df03b89d2523a53cfcb3de24025d76aba9777f4b62f0fe36c83f376

cairn gate tasks confine-release-bundle-tree-copy --root .
PASS: verdict=PASS; receipt_hash=c800aa742ad867176ce46be5edd51df42729c8632f35c9758107df0bad9766ba
```

After every task was checked, validation and all three gates passed again:

```text
cairn validate --root .
PASS: valid=true; changes=18; specs_validated=43; issues=[]

cairn gate proposal confine-release-bundle-tree-copy --root .
PASS: verdict=PASS; receipt_hash=27b9b5f7b4b40abdd43f749ce2793f7d8df7d7f76635e3efb46aa2b18e268f8c

cairn gate design confine-release-bundle-tree-copy --root .
PASS: verdict=PASS; receipt_hash=7cf9f26950d07566f0fd7704ccb1c5599032a6a5bbaa991f39b3a0e0375b565b

cairn gate tasks confine-release-bundle-tree-copy --root .
PASS: verdict=PASS; receipt_hash=7c018f6e15a29f9f648f3991abcc35c58f708f69cf1a249f13c7b2a691110bcb
```
