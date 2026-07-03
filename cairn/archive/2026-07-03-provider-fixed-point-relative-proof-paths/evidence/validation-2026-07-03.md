# Validation evidence: provider-fixed-point-relative-proof-paths

Date: 2026-07-03

## fixed-point summary path tests
```text
$ nix develop -c cargo test -p mantle --bin mantle fixed_point_summary
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.73s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 4 tests
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_explicit_complete_closure_claims ... ok
test cargo_free_self_build::tests::fixed_point_summary_records_bundle_local_stage_paths ... ok
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_provider_supplies_claim ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1099 filtered out; finished in 0.00s

```

## fixed-point preflight path tests
```text
$ nix develop -c cargo test -p mantle --bin mantle fixed_point_preflight
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test cargo_free_self_build::tests::fixed_point_preflight_records_bundle_local_execution_path ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1102 filtered out; finished in 0.00s

```

## provider fixed-point verifier compatibility tests
```text
$ nix develop -c cargo test -p mantle --bin mantle provider_fixed_point_verifier
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 5 tests
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_stage_digest_mismatch ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_enforced_closure ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_accepts_valid_bounded_bundle ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_bounded_non_claims ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rebases_copied_bundle_stage_paths ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1098 filtered out; finished in 0.00s

```

## formatting and diff checks
```text
$ nix develop -c cargo fmt --check -p mantle
$ git diff --check
```

## cairn validation and gates
```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal provider-fixed-point-relative-proof-paths --root .
{
  "change": "provider-fixed-point-relative-proof-paths",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "dfb8ff7bf73f383980174a5e9ee015ade7d9c1e40dee9b4f7ea92a630079d206",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "207e3bce9727d4be05a4fa5f12dcc429221e294f4fac29240b1a6a17934baf22",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design provider-fixed-point-relative-proof-paths --root .
{
  "change": "provider-fixed-point-relative-proof-paths",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6f8868fce4e17aacaf93b2cabee7dc247575022305521aedaf9aa4d028a37678",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4709f7d904cb7b3bb6472a389f010a3b5527a16edc49053c63eb007d2f878ae6",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks provider-fixed-point-relative-proof-paths --root .
{
  "change": "provider-fixed-point-relative-proof-paths",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "800ccf4808cf9f4a47bfe0c4347a305153f24b1fb0add3cae90ab7a6ce0927c8",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "3246d10ad0fb3b312f382a85251263f9d36091a162d1ed288516605c0006b56a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-task-completion validation
```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks provider-fixed-point-relative-proof-paths --root .
{
  "change": "provider-fixed-point-relative-proof-paths",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f7864c8632a69e6add8c2edb6c3765ee47e196a7fe51f61ca8451fda72b285cb",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ed0a78701a5703658b36fd9e198f1ae42f32b44fbdadf8ae6a4f78d485dc407e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 15,
  "valid": true
}
```
