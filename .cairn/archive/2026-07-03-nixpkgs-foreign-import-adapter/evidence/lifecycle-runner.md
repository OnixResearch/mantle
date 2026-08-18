# nixpkgs-foreign-import-adapter lifecycle evidence

Generated after the lifecycle runner archived the change but failed to write to the pre-archive evidence path. Commands below were rerun from the repository root and capture the same focused implementation checks plus post-archive Cairn validation. Gates are run by temporarily copying the archived package back under `cairn/changes/` and removing it before final status.

## Command

```text
$ nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import::
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.41s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-1e92931af0c4e32a)

running 7 tests
test foreign_derivation_import::tests::nixpkgs_hash_domain_and_frontend_metadata_fail_closed ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy ... ok
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1137 filtered out; finished in 0.00s

```

## Command

```text
$ nix develop -c cargo test -p mantle --test foreign_import_cli
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
warning: constant `HASH_DOMAIN_MISMATCH` is never used
  --> tests/foreign_import_cli.rs:29:7
   |
29 | const HASH_DOMAIN_MISMATCH: &str = "hash-domain-mismatch";
   |       ^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `mantle` (test "foreign_import_cli") generated 1 warning
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

warning: `mantle` (bin "crunch") generated 8 warnings
warning: `mantle` (bin "mantle") generated 8 warnings (8 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.08s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-ee6d0b05f48c0be4)

running 4 tests
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

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
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal nixpkgs-foreign-import-adapter --root .
{
  "change": "nixpkgs-foreign-import-adapter",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "023725892960a32ae718f1e1845cafa9d60947cd3f7b1247084e8a82f4889fd7",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "fe44495b0b73a79a7bc115c238ccdeba0e7bf6296ab30060e3ea2e982a2946da",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design nixpkgs-foreign-import-adapter --root .
{
  "change": "nixpkgs-foreign-import-adapter",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6d6a3aaf44b64758a706e6b484879d31f02ebfb1c265e0b3c78a528e80e5d622",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "636008a0eb0772808db3af0467a2267ee0758d8d6fb021f31e03dbd22652cad6",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks nixpkgs-foreign-import-adapter --root .
{
  "change": "nixpkgs-foreign-import-adapter",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "c366a54563f8631d24f89f44a11e861787ff34288b493b0f03445e66fcd5d0d2",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "8b7816bb04be8abb83897a4aa21443cda7429ff3d333af158d567fc29234489e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Command

```text
$ grep -n 'foreign_derivation_import.nixpkgs_producer_adapter\|foreign_derivation_import.nixpkgs_eval_boundary\|foreign_derivation_import.nix_hash_domain_boundary\|foreign_derivation_import.nixpkgs_substitution_first\|foreign_derivation_import.nixpkgs_receipt_non_claims' cairn/specs/foreign-derivation-import/spec.md
298:r[foreign_derivation_import.nixpkgs_producer_adapter] Nixpkgs support MUST be implemented as a producer adapter that emits `foreign-derivation-graph-v1`, `foreign-package-index-v1`, and import receipt inputs from concrete Nix derivation facts. The adapter MUST NOT make Nix expressions, flakes, overlays, or nixpkgs package-set semantics part of Mantle's stable import ABI.
317:r[foreign_derivation_import.nixpkgs_eval_boundary] Nixpkgs integration MUST keep Nix expression evaluation, flake output lookup, overlay application, and optional `snix-eval` execution in a producer shell before artifact emission. Mantle validation, planning, substitution, and realization from accepted artifacts MUST NOT invoke Nix, `nix-store`, flakes, overlays, Nix expression evaluation, package-set replacement logic, or `snix-eval`.
342:r[foreign_derivation_import.nix_hash_domain_boundary] The nixpkgs adapter MUST preserve Nix-compatible `.drv`, store-path, NAR, NARInfo, and binary-cache identity using the hash algorithms required by those formats while separately using BLAKE3 for Mantle-owned import receipts, policy digests, and translated artifact identities. The adapter MUST fail closed if a Mantle BLAKE3 digest is supplied as a Nix-compatible derivation or cache identity, or if a Nix-compatible digest is supplied where a Mantle receipt digest is required.
367:r[foreign_derivation_import.nixpkgs_substitution_first] Nixpkgs import MUST support a substitution-first compatibility level where binary-cache hints are admitted as explicit trust-scoped policy and outputs are accepted only through Mantle's normal PathInfo, NAR hash, signature, store-prefix, and attestation admission rules. Substitution-first receipts MUST NOT claim local rebuild support.
393:r[foreign_derivation_import.nixpkgs_receipt_non_claims] Nixpkgs foreign import documentation and receipts MUST distinguish admitted, planned, substituted, rebuilt, and verified states. Admission-only or planning-only receipts MUST NOT claim nixpkgs package correctness, local rebuild success, output trust, bootstrap parity, reproducibility, or future availability of the producer frontend.
```

## Command

```text
$ find cairn/archive -maxdepth 1 -type d -name '*nixpkgs-foreign-import-adapter*' -printf '%f\n' | sort
2026-07-03-nixpkgs-foreign-import-adapter
```

## Command

```text
$ git status --short --branch --untracked-files=all
## main...origin/main [ahead 18]
 M README.md
 M cairn/specs/foreign-derivation-import/spec.md
 M docs/foreign-derivation-import-trust-model.md
 M scripts/check-foreign-import-trust-model.rs
 M src/foreign_derivation_import.rs
 M src/foreign_import_cmd.rs
 M tests/foreign_import_cli.rs
?? cairn/archive/2026-07-03-nixpkgs-foreign-import-adapter/design.md
?? cairn/archive/2026-07-03-nixpkgs-foreign-import-adapter/evidence/commands.txt
?? cairn/archive/2026-07-03-nixpkgs-foreign-import-adapter/evidence/lifecycle-runner.md
?? cairn/archive/2026-07-03-nixpkgs-foreign-import-adapter/proposal.md
?? cairn/archive/2026-07-03-nixpkgs-foreign-import-adapter/specs/foreign-derivation-import/spec.md
?? cairn/archive/2026-07-03-nixpkgs-foreign-import-adapter/tasks.md
?? tests/fixtures/foreign-import/nixpkgs-hello.derivation-json.json
?? tests/fixtures/foreign-import/nixpkgs-policy.json
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-ee6d0b05f48c0be4)

running 4 tests
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

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
