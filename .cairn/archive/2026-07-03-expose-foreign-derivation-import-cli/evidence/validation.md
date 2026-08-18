# expose-foreign-derivation-import-cli validation evidence

Captured: 2026-07-03T23:19:06Z

Task-ID: V1 V2 V3 V4
Covers: foreign_derivation_import.operator_cli_surface, foreign_derivation_import.cli_file_boundary, foreign_derivation_import.checked_fixtures

## Focused binary unit tests

```text
$ nix develop -c cargo test -p mantle --bin mantle foreign_import
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2027653048753230)

running 3 tests
test foreign_import_cmd::tests::malformed_report_is_rejected ... ok
test foreign_import_cmd::tests::human_report_preserves_diagnostics_and_non_claims ... ok
test foreign_import_cmd::tests::plan_inputs_returns_receipt_and_plan_without_process_invocations ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 0.00s

```

## Focused CLI integration tests

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
   --> src/nix_free_demo_bundle.rs:118:15
    |
118 | pub(crate) fn nix_free_demo_claim(summary: &NixFreeDemoMachineSummary) -> Option<String> {
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-fc0846acab1e5d38)

running 3 tests
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

```

## Guix fixture validate smoke

```text
$ nix develop -c cargo run -q -p mantle --bin mantle -- --json foreign-import validate --graph tests/fixtures/foreign-import/guix-hello.graph.json --package-index tests/fixtures/foreign-import/guix-hello.index.json --policy tests/fixtures/foreign-import/policy.json
warning: function `child_blocker` is never used
    --> src/cargo_free_self_build.rs:1449:4
     |
1449 | fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
     |    ^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `nix_free_demo_claim` is never used
   --> src/nix_free_demo_bundle.rs:118:15
    |
118 | pub(crate) fn nix_free_demo_claim(summary: &NixFreeDemoMachineSummary) -> Option<String> {
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

{
  "schema": "mantle-foreign-import-cli-v1",
  "command": "validate",
  "verdict": "accepted",
  "accepted": true,
  "diagnostics": [],
  "receipt": {
    "schema": "foreign-derivation-import-receipt-v1",
    "producer_identity": "guix-time-machine:hello",
    "raw_graph_digest": "fccf7bfda42c24a27d41519b793f4b6253a4c4fe5783b1f43bc376f00a6838b0",
    "translation_policy_digest": "9f72d829c8d1e8acc773e7ceb535b73d3582b65c5d351661219fe718d4c70eb8",
    "translated_graph_digest": "8ddbd5581159b67f4a94e66ba5757cf79d1040733720c79c0cb2cc5dead373ff",
    "package_index_digest": "5437402da1fcda9d58dfd9c0914b0f5b4d493ad2f4424669acfff2f2c2a85fc5",
    "fetch_cache_policy_digest": "292b2c9f121b3f3e207fc74aca1b9c2e72bd188ec7ccd9c9434a4e0476577ff4",
    "sandbox_policy_digest": "b80fdec27a0955fcb9748d1ac75b7da507959fdab796a2f223c6bc8d76424afc",
    "diagnostics": [],
    "non_claims": [
      "not-build-success",
      "not-package-correctness",
      "not-bootstrap-parity",
      "not-output-trust",
      "not-reproducibility",
      "not-foreign-frontend-availability"
    ]
  },
  "plan": null,
  "non_claims": [
    "not-build-success",
    "not-package-correctness",
    "not-bootstrap-parity",
    "not-output-trust",
    "not-reproducibility",
    "not-foreign-frontend-availability"
  ]
}
```

## Nix fixture plan smoke

```text
$ nix develop -c cargo run -q -p mantle --bin mantle -- --json foreign-import plan --graph tests/fixtures/foreign-import/nix-hello.graph.json --package-index tests/fixtures/foreign-import/nix-hello.index.json --policy tests/fixtures/foreign-import/policy.json --package hello --system x86_64-linux
warning: function `child_blocker` is never used
    --> src/cargo_free_self_build.rs:1449:4
     |
1449 | fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
     |    ^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `nix_free_demo_claim` is never used
   --> src/nix_free_demo_bundle.rs:118:15
    |
118 | pub(crate) fn nix_free_demo_claim(summary: &NixFreeDemoMachineSummary) -> Option<String> {
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

{
  "schema": "mantle-foreign-import-cli-v1",
  "command": "plan",
  "verdict": "accepted",
  "accepted": true,
  "diagnostics": [],
  "receipt": {
    "schema": "foreign-derivation-import-receipt-v1",
    "producer_identity": "nix-derivation-json:hello",
    "raw_graph_digest": "0b67022339a27d76dc309024de1ac1a10d8425f5dc662b42f2505aa096127eed",
    "translation_policy_digest": "9f72d829c8d1e8acc773e7ceb535b73d3582b65c5d351661219fe718d4c70eb8",
    "translated_graph_digest": "bc1dde4d233cb85583c3adf661dfca372722369ecf1ad885e6e9d7599ab2a51f",
    "package_index_digest": "80ce2bf9825b032f4af37481eda46262b19ad950cbf058eca6f2d85bbeab9fc5",
    "fetch_cache_policy_digest": "292b2c9f121b3f3e207fc74aca1b9c2e72bd188ec7ccd9c9434a4e0476577ff4",
    "sandbox_policy_digest": "b80fdec27a0955fcb9748d1ac75b7da507959fdab796a2f223c6bc8d76424afc",
    "diagnostics": [],
    "non_claims": [
      "not-build-success",
      "not-package-correctness",
      "not-bootstrap-parity",
      "not-output-trust",
      "not-reproducibility",
      "not-foreign-frontend-availability"
    ]
  },
  "plan": {
    "schema": "mantle-foreign-derivation-adapter-plan-v1",
    "roots": [
      {
        "package_name": "hello",
        "node_id": "nix:hello",
        "output_paths": {
          "out": "/mantle/store/b2a7b75ff2ef0b905efb11b4ab6f53f9-hello"
        }
      }
    ],
    "source_payloads": [
      {
        "payload_id": "nix-hello-source",
        "kind": "fixed-output-source",
        "content_ref": "/mantle/store/00000000000000000000000000000000-hello-source",
        "embedded_text": null,
        "mirrors": [
          "https://cache.example.invalid/hello.tar.gz"
        ]
      }
    ],
    "sandbox_audit": [],
    "forbidden_process_invocations": [],
    "non_claims": [
      "not-build-success",
      "not-package-correctness",
      "not-bootstrap-parity",
      "not-output-trust",
      "not-reproducibility",
      "not-foreign-frontend-availability"
    ]
  },
  "non_claims": [
    "not-build-success",
    "not-package-correctness",
    "not-bootstrap-parity",
    "not-output-trust",
    "not-reproducibility",
    "not-foreign-frontend-availability"
  ]
}
```

## Foreign import trust-model doc guard

```text
$ nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
foreign import trust-model doc check passed
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
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 22,
  "valid": true
}
```

## Cairn proposal gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal expose-foreign-derivation-import-cli --root .
{
  "change": "expose-foreign-derivation-import-cli",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "298930d437cca97656e991e31b2dd8d1ea9c82e71627ac10475e97c818213538",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e42ca520e0b0726bd86b3d27c691ce734f61369445d12693d57b72693a13dc03",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Cairn design gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design expose-foreign-derivation-import-cli --root .
{
  "change": "expose-foreign-derivation-import-cli",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "15648caf5747682fd736c148d9e6511056c44b08746aa5f7c4e1b353e7d4becc",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "9b85196f9cfb7436ed6c1ac53c405c356dd3890a4928a99f7ec556aefd0c6117",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Cairn tasks gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks expose-foreign-derivation-import-cli --root .
{
  "change": "expose-foreign-derivation-import-cli",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1685636984c876e1a9958c5f49bad41cf1bb32f8fc6d02b94bb17d19a280f2e5",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "80ee42c847d09dc10893e0589b7780074fc773fdd95ec4daabd3b95098d4e74e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

# Accepted-spec sync evidence

Captured: 2026-07-03T23:20:49Z

Note: Cairn sync execute was run; accepted spec IDs were verified after manually merging the promoted requirement text because the full-spec-shaped delta did not insert new requirement IDs automatically.

## Cairn sync dry-run

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync expose-foreign-derivation-import-cli --root .
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/expose-foreign-derivation-import-cli/specs/foreign-derivation-import/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/expose-foreign-derivation-import-cli/specs/foreign-derivation-import/spec.md"
    }
  ],
  "blocked": false,
  "change": "expose-foreign-derivation-import-cli",
  "delta_specs": [
    "./cairn/changes/expose-foreign-derivation-import-cli/specs/foreign-derivation-import/spec.md"
  ],
  "dry_run": true,
  "input_hash": "2c78ed31602d63655952d024b592becebf3c393fba710b393f44613d8fa17576",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "41604696c4709960d3329f6ae8c24cb7120c8bee03005253d92df6ec32f4b995",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "095632c7936cd8e4ac1b3e993cb6feda1ce06c9a404f4a00bf4204d2847b4090"
}
```

## Cairn sync execute

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync expose-foreign-derivation-import-cli --root . --execute
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/expose-foreign-derivation-import-cli/specs/foreign-derivation-import/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/expose-foreign-derivation-import-cli/specs/foreign-derivation-import/spec.md"
    }
  ],
  "blocked": false,
  "change": "expose-foreign-derivation-import-cli",
  "delta_specs": [
    "./cairn/changes/expose-foreign-derivation-import-cli/specs/foreign-derivation-import/spec.md"
  ],
  "dry_run": false,
  "input_hash": "2c78ed31602d63655952d024b592becebf3c393fba710b393f44613d8fa17576",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "9e527852ffa88632245f96910ab4a4876050addb68173a96e84c26451b9f91d4",
          "exists": true,
          "path": "./cairn/specs/foreign-derivation-import/spec.md"
        }
      ],
      "manifest_hash": "4cb521dca60ea1cc29ce85a2b884e91550ac474b74b79a92788fb22291d34375"
    },
    "before": {
      "entries": [
        {
          "content_hash": "9e527852ffa88632245f96910ab4a4876050addb68173a96e84c26451b9f91d4",
          "exists": true,
          "path": "./cairn/specs/foreign-derivation-import/spec.md"
        }
      ],
      "manifest_hash": "4cb521dca60ea1cc29ce85a2b884e91550ac474b74b79a92788fb22291d34375"
    },
    "kind": "sync",
    "manifest_hash": "20fa35183ec165f516b3273c1388aa7ef7f23930636ce269f8b63fdf131a7837"
  },
  "plan_hash": "f52ee31131e0403c3c204dbf5e4ef89226724cd8c3a65610355f045dd595cd48",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "0bb4176b61be90a79accea51bbabab759fd5bddc0e645d1fe995b10ef1b6efa4"
}
```

## Accepted spec ID presence

```text
$ grep -nE 'operator_cli_surface|cli_file_boundary|checked_fixtures' cairn/specs/foreign-derivation-import/spec.md
158:r[foreign_derivation_import.operator_cli_surface] Mantle MUST provide a thin operator CLI for validating and planning from foreign derivation import artifacts without requiring the foreign frontend at consumption time.
176:r[foreign_derivation_import.cli_file_boundary] Foreign import CLI integration MUST keep filesystem reads, JSON decoding, stdout/stderr rendering, and process exit behavior in the imperative shell while preserving the translation core as pure in-memory logic.
194:r[foreign_derivation_import.checked_fixtures] Mantle MUST ship small checked-in Guix-like and Nix-like foreign import fixtures that can validate the CLI contract without live foreign frontends.
```

## Post-sync Cairn validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 22,
  "valid": true
}
```

## Post-sync tasks gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks expose-foreign-derivation-import-cli --root .
{
  "change": "expose-foreign-derivation-import-cli",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1685636984c876e1a9958c5f49bad41cf1bb32f8fc6d02b94bb17d19a280f2e5",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "80ee42c847d09dc10893e0589b7780074fc773fdd95ec4daabd3b95098d4e74e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

# Post-archive validation evidence

Captured: 2026-07-03T23:21:19Z

## Archive path

```text
cairn/archive/2026-07-03-expose-foreign-derivation-import-cli
```

## Cairn validation after archive

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
