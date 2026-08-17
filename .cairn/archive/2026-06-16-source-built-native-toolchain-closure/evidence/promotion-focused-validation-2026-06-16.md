# Promotion focused validation (2026-06-16)

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.explicit_native_promotion]

## Summary

Baseline before the core status change passed in pueue task 15: `source_toolchain_closure` reported `test result: ok. 46 passed; 0 failed; ...`, and the prior explicit-closure filter had no matching tests. After the change, the commands below passed and prove positive and negative promotion behavior.

## rustfmt-check

status: `0`

### stdout

```text
```

### stderr

```text
```

## source-tests

status: `0`

### stdout

```text

running 47 tests
test source_toolchain_closure::tests::absent_closure_status_preserves_current_non_claim ... ok
test source_toolchain_closure::tests::enforcement_rejects_undeclared_host_pkg_config_path ... ok
test source_toolchain_closure::tests::enforcement_rejects_undeclared_host_linker_path ... ok
test source_toolchain_closure::tests::enforcement_rejects_declared_path_with_digest_mismatch ... ok
test source_toolchain_closure::tests::enforcement_rejects_undeclared_host_rustc_path ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_rejects_prebuilt_policy ... ok
test source_toolchain_closure::tests::provided_rust_provider_status_promotes_source_built_claim ... ok
test source_toolchain_closure::tests::enforced_seed_exception_closure_status_keeps_non_claim ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_rejects_unknown_stage_source ... ok
test source_toolchain_closure::tests::enforced_complete_closure_status_promotes_source_built_claim ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_rejects_host_target_rustlib_aliasing ... ok
test source_toolchain_closure::tests::enforcement_accepts_declared_observed_toolchain_inputs ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_requires_ordered_stage_predecessors ... ok
test source_toolchain_closure::tests::rust_source_provider_accepts_matching_receipt_payload ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_requires_final_roles ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_bad_receipt_schema ... ok
test source_toolchain_closure::tests::json_fixture_parses_seed_exception_and_counts_source_built_members ... ok
test source_toolchain_closure::tests::policy_digest_changes_when_seed_exception_reason_changes ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_host_rustlib_satisfied_by_target_path ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_digest_mismatch_from_shell_observation ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_provenance ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_missing_target_rustlib_role ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_source_name ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_provider_receipt_digest_link_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_digest_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_rustlib_outside_lib_dir ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_artifact_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_target_rustlib_satisfied_by_host_path ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_with_prebuilt_step ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_digest_shape ... ok
test source_toolchain_closure::tests::validator_rejects_duplicate_member_identity ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_optional_receipt_on_seed_member ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_optional_source_on_seed_member ... ok
test source_toolchain_closure::tests::validated_closure_status_keeps_claim_disabled_until_enforcement_lands ... ok
test source_toolchain_closure::tests::validator_accepts_explicit_seed_exception ... ok
test source_toolchain_closure::tests::validator_allows_seed_reason_with_marker_letters_inside_larger_word ... ok
test source_toolchain_closure::tests::validator_rejects_missing_required_role ... ok
test source_toolchain_closure::tests::seed_exception_policy_digest_is_order_independent_and_accounted ... ok
test source_toolchain_closure::tests::validator_rejects_placeholder_seed_exception_reason ... ok
test source_toolchain_closure::tests::valid_manifest_yields_stable_order_independent_policy_digest ... ok
test source_toolchain_closure::tests::validator_rejects_seed_member_without_seed_exception ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_member_without_receipt ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_rustc_with_prebuilt_source_marker ... ok
test source_toolchain_closure::tests::validator_rejects_unverified_seed_exception_name ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_bootstrap_plan_yields_stable_digest ... ok
test source_toolchain_closure::tests::checked_in_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 743 filtered out; finished in 0.04s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## positive-summary

status: `0`

### stdout

```text

running 1 test
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_explicit_complete_closure_claims ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 789 filtered out; finished in 0.00s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## negative-summary

status: `0`

### stdout

```text

running 1 test
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 789 filtered out; finished in 0.00s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## diff-check

status: `0`

### stdout

```text
```

### stderr

```text
```
