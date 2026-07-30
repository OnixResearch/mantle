# Verification evidence

Date: 2026-07-30
Change: `persist-rust-unit-castore-results`

This transcript records focused implementation and performance evidence. Each command ran from the Mantle repository root.

## Implementation commits

- `b9ae7f5e` separates canonical Rust action/result identity from object presence.
- `e80ad845` adds verified local castore storage and restoration.
- `f1f5e8c9` integrates explicit `rust-plan` cache policy, receipts, retention, GC, and operator documentation.

## Focused implementation rail

Command:

```text
nix develop -c cargo test -p crunch-rust-cache-core
```

Result:

```text
running 9 tests
test tests::policy_rejects_invalid_bounds ... ok
test tests::action_rejects_unclassified_absolute_path_and_bad_digest ... ok
test tests::result_rejects_path_escape_and_tampered_reference ... ok
test tests::action_identity_is_canonical_and_excludes_output_root ... ok
test tests::local_reuse_admits_one_complete_verified_result ... ok
test tests::result_and_index_are_canonical ... ok
test tests::local_reuse_rejects_incomplete_and_mismatched_candidates ... ok
test tests::local_reuse_reports_conflicting_artifact_sets ... ok
test tests::every_declared_action_input_invalidates_identity ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 1 test
test rust_cache_core_has_no_filesystem_process_store_or_network_dependencies ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command:

```text
nix develop -c cargo test -p crunch-rust-cache
```

Result:

```text
running 10 tests
test tests::cross_filesystem_commit_facts_fail_closed ... ok
test tests::index_symlink_is_rejected_without_following ... ok
test tests::artifact_entry_bound_rejects_publication_without_result_state ... ok
test tests::publish_rejects_symlink_artifact ... ok
test tests::existing_output_blocks_restore_without_mutation ... ok
test tests::retention_plan_keeps_accepted_roots_and_prunes_only_stale_records ... ok
test tests::publish_then_restore_excludes_mutable_receipt ... ok
test tests::artifact_mismatch_cleans_interrupted_restore_staging ... ok
test tests::different_complete_results_report_conflict ... ok
test tests::incomplete_castore_candidate_is_rejected ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

Command:

```text
nix develop -c cargo test -p crunch-store \
  gc::tests::explicit_castore_root_survives_while_unreachable_blob_is_reclaimed \
  -- --exact --nocapture
```

Result:

```text
running 1 test
test gc::tests::explicit_castore_root_survives_while_unreachable_blob_is_reclaimed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 240 filtered out; finished in 0.01s
```

Command:

```text
nix develop -c cargo test -p mantle --bin mantle \
  rust_plan::tests::local_castore_ -- --nocapture
```

Result:

```text
running 2 tests
rust-local-cache-benchmark: sample=policy-fixture cold_cache_miss_us=225701 uncached_compile_us=205439 existing_output_us=2465 castore_restore_us=5994 restored_bytes=27 reused_bytes=27 compiler_invocations=2
test rust_plan::tests::local_castore_restoration_skips_second_compiler_invocation ... ok
test rust_plan::tests::local_castore_restores_every_target_unit_without_compiler_invocation ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1927 filtered out; finished in 0.46s
```

The multi-unit test deletes the complete prior execution root. It restores each unit from castore and observes no additional compiler invocation.

## Lint and compile rails

Command:

```text
cargo clippy -p crunch-rust-cache-core -p crunch-rust-cache \
  --all-targets --no-deps -- -D warnings
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.50s
```

Command:

```text
cargo test -p mantle --bin mantle --no-run
```

Result:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
Executable unittests src/main.rs
```

Broader first-party clippy remains blocked by an unrelated existing finding:

```text
src/bootstrap.rs:298:9: error: useless use of `format!`
```

`crunch-store --all-targets` clippy also reaches the unrelated existing deprecated `fetch_update` call at `crates/crunch-store/src/repair.rs:616`.

## Performance evidence

Sample: one deterministic `policy-fixture` Rust library unit.

Limits:

- Result candidates: 256.
- Tree entries: 262,144.
- Tree depth: 128.
- Tree bytes: 8,589,934,592.
- Record bytes: 4,194,304.
- Benchmark samples: 5.
- Tail statistic: maximum observed latency.

Raw output:

```text
cold_cache_miss_us=238809 uncached_compile_us=205661 existing_output_us=4040 castore_restore_us=6132 restored_bytes=27 reused_bytes=27
cold_cache_miss_us=226899 uncached_compile_us=204879 existing_output_us=2545 castore_restore_us=5141 restored_bytes=27 reused_bytes=27
cold_cache_miss_us=226969 uncached_compile_us=205434 existing_output_us=2591 castore_restore_us=4867 restored_bytes=27 reused_bytes=27
cold_cache_miss_us=226740 uncached_compile_us=206159 existing_output_us=2526 castore_restore_us=4668 restored_bytes=27 reused_bytes=27
cold_cache_miss_us=231867 uncached_compile_us=206574 existing_output_us=3775 castore_restore_us=6430 restored_bytes=27 reused_bytes=27
```

Summary:

| Route | Median (µs) | Tail max (µs) |
|---|---:|---:|
| Uncached cold compile | 205,661 | 206,574 |
| Cold local-cache miss plus compile and publication | 226,969 | 238,809 |
| Existing output-directory reuse | 2,591 | 4,040 |
| Castore restore after output deletion | 5,141 | 6,430 |
| Local-cache miss overhead over uncached compile | 22,020 | 33,148 |

Each restored sample verified and reused 27 bytes. Each restored route observed zero compiler invocations. The benchmark proves a measurable local reuse benefit for this bounded fixture. It does not prove universal workload speedup.

## Cairn validation and gates

Mantle's checked-in generated Cairn policy predates the current local Cairn binary. The lifecycle commands therefore used the current canonical policy at `/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`.

Commands:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal persist-rust-unit-castore-results --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design persist-rust-unit-castore-results --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks persist-rust-unit-castore-results --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Exact result fields:

```text
validate: valid=true changes=8 specs_validated=41
proposal: stage=proposal valid=true verdict=PASS receipt_hash=76f911396cd7f10c62b0318700470cef7375878953173f275afe5c11fe468c72
design: stage=design valid=true verdict=PASS receipt_hash=fd671d7441bc5f06844320340458f7c9152aa1d155169e834289a6625db8f608
tasks: stage=tasks valid=true verdict=PASS task_done=18 task_todo=0 receipt_hash=d04702eb45eeedd8d494e9cd8512b35d90a17449eaede3364e33f3c3e6d8419f
```

Tracey command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Exact result fields:

```text
command="traceability coverage"
input_hash=5a9d1dd4cfd0c14e92331e2a470af62dbeb550c7c4e31ffd3eae9179c79e6a14
receipt_hash=8ec6ad1b1ad34a6adce6453766d4454bfbac35bf2993db96ac6d6cccfaaf2403
referenced=263
requirements=689
valid=false
verdict=fail
error: tracey coverage failed
```

The repository-wide Tracey rail reports existing missing and dangling requirement references. The active delta is not yet part of the accepted canonical spec, and none of its `castore_result_cache` identifiers appears in either finding list. Run Tracey again after spec sync.

## Accepted-spec sync

Dry-run sync returned no blockers:

```text
mutated=false
plan_hash=a6594c4ef20c9a4c7bec5a0195dcd31bbe4c3600023447287def8b0d3d9d4923
reasons=[]
receipt_hash=1f9581b4c02fddd5c431986425bde79dccac6a37ef8f5c5da5becf74565f29ef
```

Executed sync updated `cairn/specs/rust-package-planning/spec.md`:

```text
manifest_hash=10ceb1337b06de9c4390df01d152883cb480eb778a9ffaaf7e3f9bc445c58267
plan_hash=ce4dbacdcdb1f7b23dabd8d5b87bf8bcb5fab83358aedc9dfcb22c1784efdc7a
reasons=[]
receipt_hash=0d1067a172b7abaaa264346a6836075b1e2bf57425f6559b713825771ac16736
```

Post-sync validation returned `valid=true`, `changes=8`, and `specs_validated=41`.

Post-sync Tracey reran after adding direct implementation and verification references. It remains invalid only because of repository-wide existing findings:

```text
receipt_hash=21e08988d30367d38e291345af665dd114ec51056c3e2d895675f0db8ffae6fb
referenced=270
requirements=696
valid=false
verdict=fail
```

The accepted cache contract added seven requirements and seven matching references. None of the `castore_result_cache` identifiers appears in the missing or dangling lists.

## Review checkpoint

- **Question:** Can Mantle sync and archive the local Rust unit castore result contract without treating object presence as reuse authority?
- **Inspected evidence:** Commits `b9ae7f5e`, `e80ad845`, and `f1f5e8c9`; focused core, shell, GC, integration, benchmark, and Cairn gate results above.
- **Decision:** Sync and archive this local-only cache boundary. Keep remote sharing and daemon-backed Cargo wrapper work in their separate successor changes.
- **Owner:** Mantle maintainers own the unrelated repository-wide Clippy and Tracey findings.
- **Next action:** Sync the accepted delta, rerun validation and Tracey, archive the change, and record post-archive validation.

## Non-claims

These results do not prove compiler correctness, output reproducibility, full Cargo compatibility, remote trust, release eligibility, or universal cache performance.
