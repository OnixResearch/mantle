# Validation refresh for env-scrub probe

Task-ID: validation-env-scrub-probe-2026-06-12
Covers: rust_package_planning.source_built_rust_seed_closure

## Transcript

```text
$ cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.48s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 55 tests
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test cargo_free_self_build::tests::rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback ... ok
test rust_source_provider::tests::chained_rustc_stage1_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::directory_digest_entry_limit_covers_observed_rust_source_tree ... ok
test rust_source_provider::tests::directory_validator_accepts_complete_fake_provider ... ok
test rust_source_provider::tests::directory_validator_rejects_artifact_digest_mismatch ... ok
test rust_source_provider::tests::directory_validator_rejects_nix_rustc_wrapper_even_with_matching_digest ... ok
test rust_source_provider::tests::final_chained_rustc_stage1_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::first_stage_script_fails_when_sources_removed_after_materialization ... mantle rust source first stage: mrustc-to-rust-1.90.0
missing verified source 'mrustc-0.12.0' from file:///tmp/.tmpQh3E2Q/test-source-archives/mrustc-0.12.0.tar.gz with sha256 fb3ac424dccb21825c0a8c7a7fa43982c7ffac0f4665bb1cab00c4259225072e
ok
test rust_source_provider::tests::import_provider_copies_validated_provider ... ok
test rust_source_provider::tests::import_provider_rejects_existing_output_without_overwrite ... ok
test rust_source_provider::tests::import_provider_rejects_malformed_receipt_without_output ... ok
test rust_source_provider::tests::import_provider_rejects_prebuilt_metadata_without_output ... ok
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok
test rust_source_provider::tests::materializer_rejects_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_existing_output_before_scratch_work ... ok
test rust_source_provider::tests::materializer_rejects_final_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_first_stage_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_missing_route_plan_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rustc_final_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok
test rust_source_provider::tests::rustc_final_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::rustc_stage1_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::smoke_evidence_persists_summary_output_and_metadata ... ok
test rust_source_provider::tests::smoke_evidence_rejects_tampered_smoke_output ... ok
test rust_source_provider::tests::smoke_provider_rejects_invalid_metadata_before_launch ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test source_toolchain_closure::tests::rust_source_provider_accepts_matching_receipt_payload ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_rejects_prebuilt_policy ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_rejects_unknown_stage_source ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_requires_final_roles ... ok
test source_toolchain_closure::tests::rust_source_provider_bootstrap_plan_requires_ordered_stage_predecessors ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_bad_receipt_schema ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_digest_mismatch_from_shell_observation ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_missing_target_rustlib_role ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_provenance ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_source_name ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_provider_receipt_digest_link_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_artifact_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_digest_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_with_prebuilt_step ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_rustlib_outside_lib_dir ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_bootstrap_plan_yields_stable_digest ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test tests::bootstrap_rust_source_provider_action_parses_import_dir ... ok
test tests::bootstrap_rust_source_provider_action_parses_smoke_evidence_dir ... ok
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok
test tests::bootstrap_rust_source_provider_rejects_smoke_evidence_without_smoke ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 8.72s


$ rustfmt --check src/rust_source_provider.rs
rustfmt --check ok

$ git diff --check
git diff --check ok

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1267a4c01bde9b7268e0fc16e1ad970745f1ce95145ca2e9b10f1dd3f039a57a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "65912d1a069481711a38f60d04ab9cf86fc47175899be78a460dc3212fee87f4",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-task-reference validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f93a6c32f354806d63065fa4e68732a5cf42b1596e6578fece613c3e2df8b2dd",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "776127c672cbe87af2893a033bd46bbabd2b28ec3dd3ab32ab84260c3e27a429",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
