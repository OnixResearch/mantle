# Native V2 prepublication regression: failing-before evidence

The existing `worker::tests::native_v2_late_unit_failure_keeps_slices_unpublished` regression in `crates/crunch-build/src/worker.rs:5986-6056` failed before the production repair. This is the original owner log, `/tmp/mantle-v2-prepublication-owner-tmp/red-vendor.log:595-616`, not a rerun or a claim of a passing test:

```text
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/mantle-v2-prepublication-owner-target/debug/deps/crunch_build-75212520aba46520)

running 1 test
test worker::tests::native_v2_late_unit_failure_keeps_slices_unpublished ... FAILED

failures:

---- worker::tests::native_v2_late_unit_failure_keeps_slices_unpublished stdout ----

thread 'worker::tests::native_v2_late_unit_failure_keeps_slices_unpublished' (3948829) panicked at crates/crunch-build/src/worker.rs:6041:9:
late unit admission must not publish signed slice PathInfo; registration: Err(Store("computing native dynamic output paths for 'unit.root': unable to validate output out: Invalid Store Path: Invalid length"))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    worker::tests::native_v2_late_unit_failure_keeps_slices_unpublished

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 725 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p crunch-build --lib`
OWNER_TEST_EXIT=101
```

The failing assertion queries the expected signed PathInfo before checking registration. Its failure shows that the signed slice was already visible although later unit output-path computation rejected `unit.root`. The 220-byte derivation name in this regression is one deterministic way to trigger a late validation failure, **not** the underlying defect or an acceptable special-case repair. In the current worker, `handle_completed_outcome` publishes through `admit_v2_source_slices` before `register_accepted_native_dynamic_plans` (`worker.rs:2533-2547`); unit computation and registry/goal mutation can still fail afterward (`worker.rs:756-899,2941-2963`). The test stops at its first assertion; this log does not prove downstream registry, goal, scheduler, or report state after that failure.

Accordingly T3.2 is unchecked pending V2-only pre-publication unit, registry, and goal admission and the actual passing regression plus affected scoped checks. T3.1 remains independently checked: the worker already performs verified-source signed batch publication before unit registration for accepted slices. Previously checked T3.3/T3.4/T4.1-T4.3 retain their historical bounded evidence; none is a passing-before claim for whole-plan T3.2. No implementation, archive, or final Quality gate is claimed here.

## Doc-only Cairn gates

The exact JSON stdout for the four commands below is retained in
`prepublication-cairn-gates-2026-10-03.txt`. All were run from the Mantle root
after T3.2 was unchecked; no product code, Cargo tests, or archive ran.

```text
nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
valid: true; issues: []; change_issues: []; receipt_hash: e6919fb4301d2ceb3b5fd286b9a239e8bc0b68364c5de2f2e5a18547cd680ea7
nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal add-dynamic-plan-source-slices --root .
verdict: PASS; issues: []; receipt_hash: 436d4fc800650159f79e5ce86b8fd7a81c75d7f8bab07c38311e9d91ebede59b
nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design add-dynamic-plan-source-slices --root .
verdict: PASS; issues: []; receipt_hash: 75c93edaf5aed752d5fa31b6f433509287b6e9b535efde0fddee6100e2c82dfe
nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks add-dynamic-plan-source-slices --root .
verdict: PASS; task_done: 13; task_todo: 3; task_ordering.active: false; issues: []; receipt_hash: 6a177cff2b77d9a7e0ae0747dca94c42ebfca60d363580c45f7f6a5971dfc269
```

The tasks gate does not enforce serial ordering here (`task_ordering.active:
false`), so keeping T3.2 unchecked did not require mislabeling that task or
unchecking separately evidenced downstream tasks. Gate PASS describes Cairn
document structure, **not** the failing product regression or completion of
T3.2/T4.4/T4.5.
