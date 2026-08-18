# generate-nix-free-demo-bundle validation evidence

Captured: 2026-07-03T23:31:14Z

Task-ID: V1 V2 V3
Covers: verification_evidence.nix_free_demo_bundle_generator, verification_evidence.nix_free_demo_bundle_manifest, verification_evidence.nix_free_demo_generator_non_claims

## Focused binary unit tests

```text
$ nix develop -c cargo test -p mantle --bin mantle nix_free_demo
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2027653048753230)

running 8 tests
test nix_free_demo_cmd::tests::parse_named_digest_rejects_malformed_digest ... ok
test nix_free_demo_cmd::tests::malformed_summary_report_fails_closed_with_stable_code ... ok
test nix_free_demo_bundle::tests::demo_bundle_validator_rejects_missing_guard_denial ... ok
test nix_free_demo_cmd::tests::report_for_summary_preserves_claimability_and_verdict ... ok
test nix_free_demo_bundle::tests::demo_bundle_validator_rejects_missing_fixed_point_evidence ... ok
test nix_free_demo_bundle::tests::demo_bundle_validator_accepts_matching_fixed_point_and_guard_denials ... ok
test nix_free_demo_bundle::tests::generated_readme_is_derived_from_machine_summary ... ok
test nix_free_demo_cmd::tests::generate_report_succeeds_for_claimable_bundle ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1128 filtered out; finished in 0.00s

```

## Focused CLI integration tests

```text
$ nix develop -c cargo test -p mantle --test nix_free_demo_cli -- --nocapture
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

warning: constant `BYTES_PER_KIBIBYTE` is never used
   --> src/rust_plan.rs:110:7
    |
110 | const BYTES_PER_KIBIBYTE: usize = 1024;
    |       ^^^^^^^^^^^^^^^^^^

warning: constant `RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES` is never used
   --> src/rust_plan.rs:111:7
    |
111 | const RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES: usize = 16;
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES` is never used
   --> src/rust_plan.rs:112:7
    |
112 | const RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES: usize = RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES * BYTES_PER_KIBIBYTE;
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "crunch") generated 9 warnings
warning: `mantle` (bin "mantle") generated 9 warnings (9 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running tests/nix_free_demo_cli.rs (/home/brittonr/.cargo-target/debug/deps/nix_free_demo_cli-ce01bbddce136b3d)

running 6 tests
test nix_free_demo_cli_validates_claimable_bundle_as_json ... ok
test nix_free_demo_cli_rejects_missing_fixed_point_without_success_claim ... ok
test nix_free_demo_cli_rejects_missing_guard_as_json ... ok
test nix_free_demo_cli_renders_readme_from_summary ... ok
test nix_free_demo_cli_generate_rejects_bad_inputs_without_partial_valid_bundle ... ok
test nix_free_demo_cli_generates_valid_deterministic_bundle ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

```

## Operator proof guide guard

```text
$ nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
operator proof guide drift check passed
```

## Operator proof guide self-test

```text
$ nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs --self-test
operator proof guide checker self-test passed
```

## Format check

```text
$ nix develop -c cargo fmt -p mantle --check
```

## Whitespace diff check

```text
$ git diff --check
```

## Cairn validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 21,
  "valid": true
}
```

## Cairn proposal gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal generate-nix-free-demo-bundle --root .
{
  "change": "generate-nix-free-demo-bundle",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6326be50317f5935b5eae96c1bb995780d98d322322b7c5883e1086d6ec4f054",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1ecab8e88379b26d2e306eec3283c43b6b0330cc34b2228b2150e71e15fdc29e",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Cairn design gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design generate-nix-free-demo-bundle --root .
{
  "change": "generate-nix-free-demo-bundle",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "bc766c82ea9992b3e97e33c1cc48db3e8b851a44e6f5c3abbfbcc3dcb002dfa6",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e7f5c8b69c552c7cae7b20fc9d465d5d6443ae0f786dd1cb9f29347001a11c2c",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Cairn tasks gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks generate-nix-free-demo-bundle --root .
{
  "change": "generate-nix-free-demo-bundle",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "75aec7f69fd7410692d57388105cd0a368b3072dfd9add06b6dc024aa78f5c4a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "47de9fb8ef080b2ac52c12c737143e8c29db1a14e7af1e0448aae838f43dd589",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

# Accepted-spec sync evidence

Captured: 2026-07-03T23:32:46Z

Note: Cairn sync execute was run; accepted spec IDs were verified after manually merging the promoted requirement text because the full-spec-shaped delta did not insert new requirement IDs automatically.

## Cairn sync dry-run

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync generate-nix-free-demo-bundle --root .
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/generate-nix-free-demo-bundle/specs/verification-evidence/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/generate-nix-free-demo-bundle/specs/verification-evidence/spec.md"
    }
  ],
  "blocked": false,
  "change": "generate-nix-free-demo-bundle",
  "delta_specs": [
    "./cairn/changes/generate-nix-free-demo-bundle/specs/verification-evidence/spec.md"
  ],
  "dry_run": true,
  "input_hash": "ca214a54ba68329260d527fb3eeb2e9d70757bbcab4f399e10ca01c3c72b9f04",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "024e0e264c9c4804b0c1124c44477ec69002cba12e73b201b96e6fcbf008ee10",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "4a49d4839ffeda60aa03f92ad42dd9030ee818e1da6179156fd1698d9416c6b5"
}
```

## Cairn sync execute

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync generate-nix-free-demo-bundle --root . --execute
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/generate-nix-free-demo-bundle/specs/verification-evidence/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/generate-nix-free-demo-bundle/specs/verification-evidence/spec.md"
    }
  ],
  "blocked": false,
  "change": "generate-nix-free-demo-bundle",
  "delta_specs": [
    "./cairn/changes/generate-nix-free-demo-bundle/specs/verification-evidence/spec.md"
  ],
  "dry_run": false,
  "input_hash": "ca214a54ba68329260d527fb3eeb2e9d70757bbcab4f399e10ca01c3c72b9f04",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "639d59e79cfa8172f9448ddcfe6cb2fb0857fa5416df933d908926aae84bcf08",
          "exists": true,
          "path": "./cairn/specs/verification-evidence/spec.md"
        }
      ],
      "manifest_hash": "c791d17c37cb081efef4fc804ad0a48c1d2437d3a380840d667645fe1c7c97f7"
    },
    "before": {
      "entries": [
        {
          "content_hash": "639d59e79cfa8172f9448ddcfe6cb2fb0857fa5416df933d908926aae84bcf08",
          "exists": true,
          "path": "./cairn/specs/verification-evidence/spec.md"
        }
      ],
      "manifest_hash": "c791d17c37cb081efef4fc804ad0a48c1d2437d3a380840d667645fe1c7c97f7"
    },
    "kind": "sync",
    "manifest_hash": "379534c3400888becfc6bd24436f40cfc0505b33a9d0d9700d812b4a9ca1525f"
  },
  "plan_hash": "6649f61eaf5d16aad9f66549e7a89be49c4706c8e758c087820a8ea4d2ee5f5b",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "e3294d9565856a88cacd510259a0ebad954ed72fb771f2a139459086763783bd"
}
```

## Accepted spec ID presence

```text
$ grep -nE 'nix_free_demo_bundle_generator|nix_free_demo_bundle_manifest|nix_free_demo_generator_non_claims' cairn/specs/verification-evidence/spec.md
895:r[verification_evidence.nix_free_demo_bundle_generator] Mantle SHOULD provide a generator that assembles a Nix-free demo bundle from explicit proof status, receipt digest, transcript, artifact digest, and non-claim inputs.
913:r[verification_evidence.nix_free_demo_bundle_manifest] Generated Nix-free demo bundles MUST include enough manifest data to audit status, command evidence, receipt digests, artifact digests, timestamps or run identifiers when provided, and bundle-local paths.
931:r[verification_evidence.nix_free_demo_generator_non_claims] Generated Nix-free demo bundles MUST state whether evidence is successful, blocked, synthetic, partial, or demo-only, and MUST carry explicit non-claims when evidence is not a full proof.
```

## Post-sync Cairn validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 21,
  "valid": true
}
```

## Post-sync tasks gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks generate-nix-free-demo-bundle --root .
{
  "change": "generate-nix-free-demo-bundle",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "75aec7f69fd7410692d57388105cd0a368b3072dfd9add06b6dc024aa78f5c4a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "47de9fb8ef080b2ac52c12c737143e8c29db1a14e7af1e0448aae838f43dd589",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

# Post-archive validation evidence

Captured: 2026-07-03T23:33:19Z

## Archive path

```text
cairn/archive/2026-07-03-generate-nix-free-demo-bundle
```

## Cairn validation after archive

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 20,
  "valid": true
}
```
