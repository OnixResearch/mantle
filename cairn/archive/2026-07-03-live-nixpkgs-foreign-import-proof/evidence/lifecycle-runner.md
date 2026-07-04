# live-nixpkgs-foreign-import-proof lifecycle evidence

Generated manually because `scripts/cairn-lifecycle-evidence.rs` writes its evidence file after archive execution, which removes the active change path. Commands below are run from the repository root.

## Command

```text
$ test -f cairn/changes/live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/summary.md
```

## Command

```text
$ grep -q 'Validate accepted: true' cairn/changes/live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/summary.md
```

## Command

```text
$ grep -q 'Plan accepted: true' cairn/changes/live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/summary.md
```

## Command

```text
$ grep -q 'Plan forbidden process invocations: 0' cairn/changes/live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/summary.md
```

## Command

```text
$ grep -q 'no local rebuild compatibility' cairn/changes/live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/summary.md
```

## Command

```text
$ nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import::
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.31s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-1e92931af0c4e32a)

running 9 tests
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_missing_output_path_without_env_fallback ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_normalizes_relative_paths_and_fod_env_outputs ... ok
test foreign_derivation_import::tests::nixpkgs_hash_domain_and_frontend_metadata_fail_closed ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1137 filtered out; finished in 0.00s

```

## Command

```text
$ nix develop -c cargo test -p mantle --test foreign_import_cli
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: function `child_blocker` is never used
    --> src/cargo_free_self_build.rs:1449:4
     |
1449 | fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
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

warning: `mantle` (bin "mantle") generated 8 warnings
warning: `mantle` (bin "crunch") generated 8 warnings (8 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-ee6d0b05f48c0be4)

running 4 tests
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

```

## Command

```text
$ nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
foreign import trust-model doc check passed
```

## Command

```text
$ nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test
foreign import trust-model checker self-test passed
```

## Command

```text
$ nix develop -c cargo fmt --check -p mantle
```

## Command

```text
$ git diff --check
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
waiting for another Nix process to finish fetching input 'path:/home/brittonr/git/cairn'...
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 17,
  "valid": true
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal live-nixpkgs-foreign-import-proof --root .
waiting for another Nix process to finish fetching input 'path:/home/brittonr/git/cairn'...
{
  "change": "live-nixpkgs-foreign-import-proof",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "aa8a6f768e3ff3a25189faa81b551b6871d292fb74039762873e90bb20206428",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4e2f22f4ff074bdefbb2f66b883b58ea4e6699e21fff39b55a2b77007f858351",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design live-nixpkgs-foreign-import-proof --root .
waiting for another Nix process to finish fetching input 'path:/home/brittonr/git/cairn'...
{
  "change": "live-nixpkgs-foreign-import-proof",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b8a713e48a3ae1d4f6719b94d1ced6ef84cda3d8d79e214f4fdcabcf35bac6f5",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ea0c99cbd11d8d9a3c7a12648a68d8045abdf6f59135b85d21a8f1158cf5204f",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks live-nixpkgs-foreign-import-proof --root .
{
  "change": "live-nixpkgs-foreign-import-proof",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "7eb1fe415115fd8616c971585646c2a89aac05841984b63c36c5011684b5277e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "d82b46cb31ffb7733a783fad165ea634451cc7bb65ffec35e62fa461f2ca2b12",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync live-nixpkgs-foreign-import-proof --root .
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/live-nixpkgs-foreign-import-proof/specs/foreign-derivation-import/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/live-nixpkgs-foreign-import-proof/specs/foreign-derivation-import/spec.md"
    }
  ],
  "blocked": false,
  "change": "live-nixpkgs-foreign-import-proof",
  "delta_specs": [
    "./cairn/changes/live-nixpkgs-foreign-import-proof/specs/foreign-derivation-import/spec.md"
  ],
  "dry_run": true,
  "input_hash": "50dd4662e00f9b3e713b915d6732f5deb1600a0ed5731bf07c21844407333adc",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "681f03845dd1dda7f25dd7eea30effecba6764b7890986f134bf7d26ad829da5",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "b3e1b35d815f9f234601a20253c2b00e88f687d9b6a3da81645c189d4d4df32f"
}
```

## Command

```text
$ grep -n 'foreign_derivation_import.live_nixpkgs_export_to_plan_proof' cairn/specs/foreign-derivation-import/spec.md
411:r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof] Mantle SHOULD maintain current evidence that a real Nixpkgs package can be exported through host Nix into concrete derivation facts, lowered into foreign import artifacts, and consumed by Mantle validation/planning without Nix available during consumption. The proof MUST name the strongest proven state and MUST NOT claim substitution, local rebuild compatibility, output trust, package correctness, or reproducibility.
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}
```

## Command

```text
$ find cairn/archive -maxdepth 1 -type d -name '*live-nixpkgs-foreign-import-proof*' -printf '%f\n' | sort
2026-07-03-live-nixpkgs-foreign-import-proof
```

## Command

```text
$ git status --short --branch --untracked-files=all
## main...origin/main [ahead 19]
 M cairn/specs/foreign-derivation-import/spec.md
 M docs/foreign-derivation-import-trust-model.md
 M src/foreign_derivation_import.rs
 M src/foreign_import_cmd.rs
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/design.md
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/commands.txt
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/lifecycle-runner.md
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/artifacts/nixpkgs.graph.json
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/artifacts/nixpkgs.index.json
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/derivation-json.json
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/nixpkgs-metadata.txt
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/plan-report.json
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/produce-report.json
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/summary.md
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/transcript.md
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/validate-report.json
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/proposal.md
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/specs/foreign-derivation-import/spec.md
?? cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/tasks.md
```
