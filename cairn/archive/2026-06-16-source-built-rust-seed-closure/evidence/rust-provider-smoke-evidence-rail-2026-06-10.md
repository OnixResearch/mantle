# Rust provider smoke evidence rail

Task-ID: rust-provider-smoke-evidence-rail-2026-06-10
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now accepts `bootstrap rust-source-provider --smoke --smoke-evidence-dir <dir>`. When a validated provider is supplied or materialized in the future, the smoke rail writes durable smoke evidence next to the operator-selected bundle: summary JSON, stdout, stderr, smoke source, smoke output artifact, and copied provider metadata. The evidence writer checks BLAKE3 digests for the copied output artifact and provider metadata before writing the summary.

This does not complete the real-provider smoke task: no real source-built Rust provider exists yet, so the task remains blocked. The change only makes the future provider-backed smoke proof record durable artifacts instead of tempdir-only diagnostics.

## Commands and output

### provider smoke/evidence tests

Command:

```sh
cargo test -p mantle --bin mantle rust_source_provider
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 33 tests
test cargo_free_self_build::tests::rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_provenance ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_source_name ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_missing_target_rustlib_role ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_provider_receipt_digest_link_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_digest_mismatch_from_shell_observation ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_digest_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_bad_receipt_schema ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_artifact_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_with_prebuilt_step ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_rustlib_outside_lib_dir ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test source_toolchain_closure::tests::rust_source_provider_accepts_matching_receipt_payload ... ok
test rust_source_provider::tests::materializer_fails_closed_without_claiming_prebuilt_rust ... ok
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok
test rust_source_provider::tests::import_provider_rejects_existing_output_without_overwrite ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test rust_source_provider::tests::directory_validator_rejects_artifact_digest_mismatch ... ok
test rust_source_provider::tests::import_provider_rejects_malformed_receipt_without_output ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test rust_source_provider::tests::directory_validator_accepts_complete_fake_provider ... ok
test rust_source_provider::tests::smoke_provider_rejects_invalid_metadata_before_launch ... ok
test rust_source_provider::tests::import_provider_rejects_prebuilt_metadata_without_output ... ok
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test tests::bootstrap_rust_source_provider_action_parses_smoke_evidence_dir ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test tests::bootstrap_rust_source_provider_rejects_smoke_evidence_without_smoke ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::bootstrap_rust_source_provider_action_parses_import_dir ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test rust_source_provider::tests::import_provider_copies_validated_provider ... ok
test rust_source_provider::tests::smoke_evidence_rejects_tampered_smoke_output ... ok
test rust_source_provider::tests::smoke_evidence_persists_summary_output_and_metadata ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 0.02s


```

### bootstrap rust-source-provider CLI tests

Command:

```sh
cargo test -p mantle --bin mantle bootstrap_rust_source_provider
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 5 tests
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok
test tests::bootstrap_rust_source_provider_rejects_smoke_evidence_without_smoke ... ok
test tests::bootstrap_rust_source_provider_action_parses_smoke_evidence_dir ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test tests::bootstrap_rust_source_provider_action_parses_import_dir ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 746 filtered out; finished in 0.00s


```

### cairn validate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . 
```

Exit status: `0`

```text
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

```

### cairn tasks gate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root . 
```

Exit status: `0`

```text
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
  "input_hash": "ae391fd6c1751184eb9168268187303326e1eb1e1ddc6102b72552b4ae3189b1",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "70819d3e7ce01b71dbcbe0fc34a15108e1f88d4c4c70b2d8e5f427979725a45c",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

