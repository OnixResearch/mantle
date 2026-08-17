# offline-build-operator-runbook lifecycle evidence

Generated as a repair transcript after `scripts/cairn-lifecycle-evidence.rs` executed the change and archived it, then failed to write its active-path transcript because `cairn/changes/offline-build-operator-runbook` had already moved to `cairn/archive/2026-07-04-offline-build-operator-runbook`.


## Command

```text
$ sh -c test ! -e cairn/changes/offline-build-operator-runbook
```

## Command

```text
$ nix develop -c cargo fmt --check -p mantle -v
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_compare.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_eval_backends.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_eval_smoke.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_lazy_eval.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_suite.rs"
[lib (2024)] "/home/brittonr/git/mantle/src/lib.rs"
[bin (2024)] "/home/brittonr/git/mantle/src/main.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/attest_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/audit_support.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/benchmark_harness.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_eval.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_parity_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_validate_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/cargo_free_self_build_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/cargo_import_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/examples_build.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/examples_eval.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/examples_inventory.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/foreign_import_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/identity_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/integration.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/integration_build.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/nix_free_demo_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/offline_build_runbook_docs.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/offline_cargo_project.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/operator_diagnostics.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/pin_import_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_build_smoke.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_refresh_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/release_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/remote_stdio_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/removed_system_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/rust_compatibility_rail.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/rust_plan_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/self_hosting.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/smoke.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/source_bundle_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/stdlib_tests.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/store_archive_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/store_gc_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/transcript_cli.rs"
rustfmt --edition 2024 --check /home/brittonr/git/mantle/examples/benchmark_compare.rs /home/brittonr/git/mantle/examples/benchmark_eval_backends.rs /home/brittonr/git/mantle/examples/benchmark_eval_smoke.rs /home/brittonr/git/mantle/examples/benchmark_lazy_eval.rs /home/brittonr/git/mantle/examples/benchmark_suite.rs /home/brittonr/git/mantle/src/lib.rs /home/brittonr/git/mantle/src/main.rs /home/brittonr/git/mantle/tests/attest_cli.rs /home/brittonr/git/mantle/tests/audit_support.rs /home/brittonr/git/mantle/tests/benchmark_harness.rs /home/brittonr/git/mantle/tests/bootstrap_eval.rs /home/brittonr/git/mantle/tests/bootstrap_parity_cli.rs /home/brittonr/git/mantle/tests/bootstrap_validate_cli.rs /home/brittonr/git/mantle/tests/cargo_free_self_build_cli.rs /home/brittonr/git/mantle/tests/cargo_import_cli.rs /home/brittonr/git/mantle/tests/examples_build.rs /home/brittonr/git/mantle/tests/examples_eval.rs /home/brittonr/git/mantle/tests/examples_inventory.rs /home/brittonr/git/mantle/tests/foreign_import_cli.rs /home/brittonr/git/mantle/tests/identity_cli.rs /home/brittonr/git/mantle/tests/integration.rs /home/brittonr/git/mantle/tests/integration_build.rs /home/brittonr/git/mantle/tests/nix_free_demo_cli.rs /home/brittonr/git/mantle/tests/offline_build_runbook_docs.rs /home/brittonr/git/mantle/tests/offline_cargo_project.rs /home/brittonr/git/mantle/tests/operator_diagnostics.rs /home/brittonr/git/mantle/tests/pin_import_cli.rs /home/brittonr/git/mantle/tests/project_build_smoke.rs /home/brittonr/git/mantle/tests/project_cli.rs /home/brittonr/git/mantle/tests/project_refresh_cli.rs /home/brittonr/git/mantle/tests/release_cli.rs /home/brittonr/git/mantle/tests/remote_stdio_cli.rs /home/brittonr/git/mantle/tests/removed_system_cli.rs /home/brittonr/git/mantle/tests/rust_compatibility_rail.rs /home/brittonr/git/mantle/tests/rust_plan_cli.rs /home/brittonr/git/mantle/tests/self_hosting.rs /home/brittonr/git/mantle/tests/smoke.rs /home/brittonr/git/mantle/tests/source_bundle_cli.rs /home/brittonr/git/mantle/tests/stdlib_tests.rs /home/brittonr/git/mantle/tests/store_archive_cli.rs /home/brittonr/git/mantle/tests/store_gc_cli.rs /home/brittonr/git/mantle/tests/transcript_cli.rs
```

## Command

```text
$ nix develop -c cargo test -p mantle --bin mantle source_offline_preflight -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling ring v0.17.14
   Compiling rustls v0.23.37
   Compiling rustls-webpki v0.103.13
   Compiling tokio-rustls v0.26.4
   Compiling rustls-platform-verifier v0.6.2
   Compiling ureq v3.3.0
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling object_store v0.14.0
   Compiling gix-transport v0.55.1
   Compiling reqwest-tracing v0.6.0
   Compiling gix-protocol v0.59.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/mantle/vendor/snix-tracing)
   Compiling gix v0.81.0
   Compiling snix-castore v0.1.0 (/home/brittonr/git/mantle/vendor/snix-castore)
   Compiling snix-store v0.1.0 (/home/brittonr/git/mantle/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/mantle/crates/crunch-store)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/mantle/crates/crunch-delta)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 34.54s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-1e92931af0c4e32a)

running 7 tests
test source_bundle::tests::source_offline_preflight_allows_roots_without_source_requirements ... ok
test source_bundle::tests::source_offline_preflight_reports_remote_network_requirement ... ok
test source_bundle::tests::source_offline_preflight_reports_missing_local_source_state ... ok
test source_bundle::tests::source_offline_preflight_reports_unsupported_adapter ... ok
test source_bundle::tests::source_offline_preflight_rejects_unpinned_imported_source ... ok
test source_bundle::tests::source_offline_preflight_reports_untrusted_adapter ... ok
test source_bundle::tests::source_offline_preflight_accepts_pinned_imported_file_payload ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1182 filtered out; finished in 0.01s

```

## Command

```text
$ nix develop -c cargo test -p mantle --bin mantle offline_cargo_evidence -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-1e92931af0c4e32a)

running 3 tests
test build_report::tests::build_json_report_omits_missing_offline_cargo_evidence_sidecar ... ok
test build_report::tests::build_json_report_diagnoses_malformed_offline_cargo_evidence_sidecar ... ok
test build_report::tests::build_json_report_surfaces_offline_cargo_evidence_sidecar ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s

```

## Command

```text
$ nix develop -c cargo test -p mantle --test source_bundle_cli source_bundle_cli_preflight -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
warning: function `child_blocker` is never used
    --> src/cargo_free_self_build.rs:1557:4
     |
1557 | fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
     |    ^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `nix_free_demo_claim` is never used
   --> src/nix_free_demo_bundle.rs:216:15
    |
216 | pub(crate) fn nix_free_demo_claim(summary: &NixFreeDemoMachineSummary) -> Option<String> {
    |               ^^^^^^^^^^^^^^^^^^^

warning: constant `TRUST_WINDOW_START_UNIX_S` is never used
  --> src/portable_receipt.rs:29:7
   |
29 | const TRUST_WINDOW_START_UNIX_S: u64 = 0;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `build_receipt_bundle` is never used
   --> src/portable_receipt.rs:226:8
    |
226 | pub fn build_receipt_bundle(
    |        ^^^^^^^^^^^^^^^^^^^^

warning: function `verify_receipt_bundle_against_state` is never used
   --> src/portable_receipt.rs:324:8
    |
324 | pub fn verify_receipt_bundle_against_state(
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `verify_receipt_bundle_against_archive_report` is never used
   --> src/portable_receipt.rs:353:8
    |
353 | pub fn verify_receipt_bundle_against_archive_report(
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `route_plan_for_existing_build_action` is never used
   --> src/realization_routing.rs:470:8
    |
470 | pub fn route_plan_for_existing_build_action(action: &str, detail: Option<&str>) -> RoutePlanReport {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES` is never used
   --> src/rust_plan.rs:121:7
    |
121 | const RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES: usize = 16;
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES` is never used
   --> src/rust_plan.rs:122:7
    |
122 | const RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES: usize = RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES * BYTES_PER_KIBIBYTE;
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "crunch") generated 9 warnings
warning: `mantle` (bin "mantle") generated 9 warnings (9 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.21s
     Running tests/source_bundle_cli.rs (/home/brittonr/.cargo-target/debug/deps/source_bundle_cli-6ad4787f9cff0bc4)

running 2 tests
test source_bundle_cli_preflight_reports_network_required_before_build ... ok
test source_bundle_cli_preflight_reports_unpinned_imported_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.13s

```

## Command

```text
$ nix develop -c cargo test -p mantle --test offline_build_runbook_docs -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
warning: function `child_blocker` is never used
    --> src/cargo_free_self_build.rs:1557:4
     |
1557 | fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
     |    ^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `nix_free_demo_claim` is never used
   --> src/nix_free_demo_bundle.rs:216:15
    |
216 | pub(crate) fn nix_free_demo_claim(summary: &NixFreeDemoMachineSummary) -> Option<String> {
    |               ^^^^^^^^^^^^^^^^^^^

warning: constant `TRUST_WINDOW_START_UNIX_S` is never used
  --> src/portable_receipt.rs:29:7
   |
29 | const TRUST_WINDOW_START_UNIX_S: u64 = 0;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `build_receipt_bundle` is never used
   --> src/portable_receipt.rs:226:8
    |
226 | pub fn build_receipt_bundle(
    |        ^^^^^^^^^^^^^^^^^^^^

warning: function `verify_receipt_bundle_against_state` is never used
   --> src/portable_receipt.rs:324:8
    |
324 | pub fn verify_receipt_bundle_against_state(
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `verify_receipt_bundle_against_archive_report` is never used
   --> src/portable_receipt.rs:353:8
    |
353 | pub fn verify_receipt_bundle_against_archive_report(
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `route_plan_for_existing_build_action` is never used
   --> src/realization_routing.rs:470:8
    |
470 | pub fn route_plan_for_existing_build_action(action: &str, detail: Option<&str>) -> RoutePlanReport {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES` is never used
   --> src/rust_plan.rs:121:7
    |
121 | const RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES: usize = 16;
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES` is never used
   --> src/rust_plan.rs:122:7
    |
122 | const RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES: usize = RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES * BYTES_PER_KIBIBYTE;
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "mantle") generated 9 warnings
warning: `mantle` (bin "crunch") generated 9 warnings (9 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.45s
     Running tests/offline_build_runbook_docs.rs (/home/brittonr/.cargo-target/debug/deps/offline_build_runbook_docs-a90e7428a94a0d1e)

running 3 tests
test offline_build_runbook_validator_rejects_missing_commands_and_overclaims ... ok
test offline_build_runbook_docs_cover_commands_evidence_and_non_claims ... ok
test cli_help_exposes_source_bundle_and_offline_source_preflight ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 8,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 26,
  "valid": true
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal offline-build-operator-runbook --root .
{
  "change": "offline-build-operator-runbook",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "2f07c287505d04c9f712666c4edbc3b9b638ec3d4ee459d7dec4c0dc95caa04b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c67f498f06b1e01952dc91feb0d413dd956945b6d5d22c4f9520cea35197fcd0",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design offline-build-operator-runbook --root .
{
  "change": "offline-build-operator-runbook",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "fe95aa04d63987e9b9f0bbc13ad1114e61c20c8cd2916498891dc80380db688c",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "58614f2eeb632f5358ab9f04dd2b026bacec3fcc5040a8551b61fdf53e31338e",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks offline-build-operator-runbook --root .
{
  "change": "offline-build-operator-runbook",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "abc9cfe178377cfacd906a3ffd5eb8904558230c0eb4af79cf73913811271e8f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c492c7c76117a7cd6c38a29584c64473708e9df3f8d1756a5fc556f248acf605",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 25,
  "valid": true
}
```

## Command

```text
$ grep -n operator_diagnostics.offline_build_runbook cairn/specs/operator-diagnostics/spec.md
115:r[operator_diagnostics.offline_build_runbook] Mantle MUST document and diagnose the offline build workflow as an explicit, claim-bounded operator path. The runbook and diagnostics MUST cover source-bundle export, source-bundle import with pinning, source-bundle verify or offline preflight, `mantle build --offline-source-preflight --no-substitute`, evidence inspection, and common blocker remediation without presenting source readiness, route eligibility, or offline Cargo evidence as build success or stronger proof than the underlying evidence supports.
```

## Command

```text
$ sh -c test -d cairn/archive/2026-07-04-offline-build-operator-runbook && test ! -e cairn/changes/offline-build-operator-runbook
```

## Command

```text
$ git status --short --branch
## main...origin/main [ahead 2]
 M README.md
 M cairn/specs/operator-diagnostics/spec.md
 M docs/operator-proof-guide.md
 M docs/operator-workflows.md
 M examples/README.md
 M src/build_report.rs
 M src/source_bundle.rs
 M tests/source_bundle_cli.rs
?? cairn/archive/2026-07-04-offline-build-operator-runbook/
?? cairn/changes/cache-substitution-reuse-diagnostics/
?? cairn/changes/offline-bootstrap-source-bundles/
?? cairn/changes/offline-cargo-evidence-v2/
?? cairn/changes/offline-cargo-vendor-import/
?? cairn/changes/rust-compatibility-rail-expansion/
?? cairn/changes/source-bundle-realization-route/
?? tests/offline_build_runbook_docs.rs
```
