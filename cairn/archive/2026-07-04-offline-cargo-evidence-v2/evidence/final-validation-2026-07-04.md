# Final validation for offline-cargo-evidence-v2

Task-ID: offline-cargo-evidence-v2-final-validation
Covers: project_workflows.offline_cargo_digest_bound_evidence


## nix develop -c cargo fmt --check -p mantle

```text

```

## nix develop -c cargo test -p mantle --bin mantle offline_cargo::tests::

```text

running 5 tests
test offline_cargo::tests::valid_offline_cargo_plan_records_bounded_claims ... ok
test offline_cargo::tests::digest_bound_evidence_accepts_content_and_store_path_identities ... ok
test offline_cargo::tests::invalid_offline_cargo_plan_fails_closed_without_network_fallback ... ok
test offline_cargo::tests::stale_offline_cargo_digests_fail_closed_before_accepting_outputs ... ok
test offline_cargo::tests::digest_bound_evidence_rejects_stale_or_impure_claims ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1190 filtered out; finished in 0.01s


```

## nix develop -c cargo test -p mantle --bin mantle build_report::tests::

```text

running 13 tests
test build_report::tests::count_as_u32_round_trips_small_values ... ok
test build_report::tests::build_counts_splits_cached_and_built ... ok
test build_report::tests::build_json_report_omits_missing_offline_cargo_evidence_sidecar ... ok
test build_report::tests::success_log_file_is_none_when_log_is_missing ... ok
test build_report::tests::build_json_report_diagnoses_malformed_offline_cargo_evidence_sidecar ... ok
test build_report::tests::render_build_json_report_serializes_full_substitution_fields_stably ... ok
test build_report::tests::build_json_report_includes_artifact_attestation_reference ... ok
test build_report::tests::build_json_preflight_failure_omits_saved_log_path_field ... ok
test build_report::tests::render_build_json_report_serializes_frontend_artifact_attestation ... ok
test build_report::tests::build_json_report_surfaces_legacy_offline_cargo_evidence_sidecar ... ok
test build_report::tests::build_json_failure_report_uses_typed_envelope_schema ... ok
test build_report::tests::build_json_report_diagnoses_stale_digest_bound_offline_cargo_evidence_sidecar ... ok
test build_report::tests::build_json_report_surfaces_digest_bound_offline_cargo_evidence_sidecar ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1182 filtered out; finished in 0.07s


```

## nix develop -c cargo test -p mantle --test offline_cargo_project

```text

running 3 tests
test offline_cargo_project_run_executes_declared_sandbox_cargo_output ... ok
test offline_cargo_helper_lowers_to_derivation_with_bounded_evidence ... ok
test offline_cargo_helper_rejects_missing_binary_name ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.13s


```

## grep -n "digest-bound" README.md docs/operator-workflows.md

```text
README.md:1484:output is materialized locally. Current v2 evidence is digest-bound: reports
docs/operator-workflows.md:264:`cargo-inside-mantle-sandbox` plus digest-bound Cargo.lock, package-source,
docs/operator-workflows.md:649:`mantle-global-reproducibility-report-v1` for a digest-bound universe and

```

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle

```text
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 22,
  "valid": true
}

```

## nix run path:/home/brittonr/git/cairn#cairn -- gate proposal offline-cargo-evidence-v2 --root /home/brittonr/git/mantle

```text
{
  "change": "offline-cargo-evidence-v2",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "35e991843ba502294b6fb89581df37c6a457d459bc093367f3d46b4d698c3898",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "3080eb20525e5e01132c14e975d10a0ae860a72321570f78879ce1e148d328c5",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## nix run path:/home/brittonr/git/cairn#cairn -- gate design offline-cargo-evidence-v2 --root /home/brittonr/git/mantle

```text
{
  "change": "offline-cargo-evidence-v2",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6ec7bbe43d9432fc7e253040ef9b8f6600abdc2f02b56555512b575a6576a5da",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "55a08d4bb92500924c1b059cd8dcb00834bae8483753ee6e4a825951b9786b9a",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## nix run path:/home/brittonr/git/cairn#cairn -- gate tasks offline-cargo-evidence-v2 --root /home/brittonr/git/mantle

```text
{
  "change": "offline-cargo-evidence-v2",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "5ff0ba5478a9a0bcba2df7e66b14081e1a8a1dad17d84b8fe34e7279653e4071",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "31a84da8a752f5324ee0401b1e594c0fc5cf9c892ad6f85281c7157c18e7a224",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

## grep -n "project_workflows.offline_cargo_digest_bound_evidence" cairn/specs/project-workflows/spec.md

```text
510:r[project_workflows.offline_cargo_digest_bound_evidence] Mantle MUST emit and report versioned offline Cargo package evidence that binds the Cargo action to declared offline inputs by digest. The evidence MUST record the package source identity, Cargo.lock BLAKE3 digest, optional vendored dependency source identity, selected Rust/toolchain inputs, target, profile, Cargo command shape, network policy result, output identity, and bounded non-claims. Malformed or stale evidence MUST be diagnosed rather than silently promoted to a successful offline Cargo evidence claim.

```

## CAIRN_ARCHIVE_DATE=2026-07-04 nix run path:/home/brittonr/git/cairn#cairn -- archive offline-cargo-evidence-v2 --root /home/brittonr/git/mantle --execute

```text
archive_path=cairn/archive/2026-07-04-offline-cargo-evidence-v2
```

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle (post-archive)

```text
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 21,
  "valid": true
}
```

## accepted requirement after archive

```text
510:r[project_workflows.offline_cargo_digest_bound_evidence] Mantle MUST emit and report versioned offline Cargo package evidence that binds the Cargo action to declared offline inputs by digest. The evidence MUST record the package source identity, Cargo.lock BLAKE3 digest, optional vendored dependency source identity, selected Rust/toolchain inputs, target, profile, Cargo command shape, network policy result, output identity, and bounded non-claims. Malformed or stale evidence MUST be diagnosed rather than silently promoted to a successful offline Cargo evidence claim.
```
