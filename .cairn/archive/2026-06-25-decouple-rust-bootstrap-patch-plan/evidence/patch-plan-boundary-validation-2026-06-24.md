# Patch-plan boundary validation — 2026-06-24

## Scope

Task IDs: I1, I2, I3, I4, V1, V2

Covers:
- r[rust_package_planning.source_built_toolchain_closure.bootstrap_patch_plan_boundary]
- r[rust_package_planning.source_built_toolchain_closure.provider_contract_independence]

## Implementation evidence

- Added `src/rust_bootstrap_patch_plan.rs` as a deterministic functional core. It derives ordered Rust bootstrap repair operations from explicit route facts, source identities, compiler versions, triples, and route capabilities. It computes BLAKE3 input/output digests over serialized plan facts and operations.
- Kept provider materialization in `src/rust_source_provider.rs` as the imperative shell. Existing source/makefile/wrapper mutations are dispatched through operation kinds, and unsupported operation kinds fail closed before writing shell fragments.
- Recorded `patch_plan` objects in first-stage, Rust stage1, and final Rust build/plan manifests. Provider receipts now include a `record-rust-bootstrap-patch-plan` step with schema, stage/route/triple/version facts, input/output digests, operation count, and operation summaries.
- Added provider-contract independence coverage that checks native planning/topology/self-build source files do not branch on `mrustc`, `minicargo`, or `run_rustc` implementation details.

## Validation transcript

### Focused patch-plan core tests

Command: pueue task 1150

```text
cargo test -p mantle --bin mantle rust_bootstrap_patch_plan::tests:: -- --nocapture --test-threads=1

running 7 tests
test rust_bootstrap_patch_plan::tests::musl_host_first_stage_plan_derives_ordered_operations_and_stable_digest ... ok
test rust_bootstrap_patch_plan::tests::non_musl_first_stage_plan_omits_musl_route_repairs ... ok
test rust_bootstrap_patch_plan::tests::patch_plan_rejects_mismatched_first_stage_versions ... ok
test rust_bootstrap_patch_plan::tests::patch_plan_rejects_missing_musl_runtime_capability ... ok
test rust_bootstrap_patch_plan::tests::patch_plan_rejects_missing_source_identity_before_shell_mutation ... ok
test rust_bootstrap_patch_plan::tests::patch_plan_rejects_unsupported_bootstrap_version ... ok
test rust_bootstrap_patch_plan::tests::rust_bootstrap_plan_includes_final_rustdoc_rlib_lookup_repair ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 822 filtered out; finished in 0.00s
```

### Focused provider tests

Command: pueue task 1144

```text
cargo test -p mantle --bin mantle rust_source_provider::tests:: -- --nocapture --test-threads=1

test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok
test rust_source_provider::tests::musl_target_tool_candidates_include_source_root_aliases_only ... ok
test rust_source_provider::tests::rust_bootstrap_patch_operation_dispatch_rejects_unsupported_operation ... ok
test rust_source_provider::tests::rustc_final_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::rustc_stage1_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_copies_shared_libgcc_runtime ... ok
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_rejects_missing_shared_libgcc_runtime ... ok
test rust_source_provider::tests::smoke_evidence_persists_summary_output_and_metadata ... ok
test rust_source_provider::tests::smoke_evidence_rejects_tampered_smoke_output ... ok
test rust_source_provider::tests::smoke_provider_rejects_invalid_metadata_before_launch ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test rust_source_provider::tests::smoke_rustc_args_do_not_duplicate_provider_wrapper_sysroot ... ok

test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 783 filtered out; finished in 15.49s
```

### Formatting, diff, and Cairn gates

Command: pueue task 1141

```text
/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check --config skip_children=true src/main.rs
/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_bootstrap_patch_plan.rs src/rust_source_provider.rs
git diff --check
cargo test -p mantle --bin mantle rust_bootstrap_patch_plan::tests:: -- --nocapture
cargo test -p mantle --bin mantle rust_source_provider::tests:: -- --nocapture
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal decouple-rust-bootstrap-patch-plan --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design decouple-rust-bootstrap-patch-plan --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks decouple-rust-bootstrap-patch-plan --root .

{
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Note: `rustfmt --check src/main.rs` without `skip_children=true` traverses unrelated root modules and reports existing `src/build_report.rs` formatting churn. The validation rail checked the touched crate root with `skip_children=true` and checked the changed leaf files directly.

## Remaining proof

Post-task evidence update validation: pueue task 1153 reran `git diff --check`, `cairn validate --root .`, and proposal/design/tasks gates. Final tasks gate JSON reported `"issues": []`, `"valid": true`, and `"verdict": "PASS"` with receipt hash `8e8903d2613befcf0aafa5d8817b835426379d1f1c49e9eaf3aebac8596944ca`.

V3 remains open. A real source-built Rust provider rerun must be queued or captured from committed refactor code before this change is archived.
