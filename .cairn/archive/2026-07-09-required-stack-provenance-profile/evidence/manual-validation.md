# required-stack-provenance-profile validation transcript
2026-07-09T18:36:02Z
## focused commands
```text
$ nix develop -c rustfmt --check --config skip_children=true crates/crunch-release-core/src/lib.rs crates/crunch-release-core/src/manifest.rs src/main.rs src/release_cmd.rs
```
```text
$ nix develop -c cargo test -p crunch-release-core release_profile --lib

running 2 tests
test manifest::tests::release_profile_maps_generic_and_onix_stack_modes ... ok
test manifest::tests::release_profile_rejects_unsupported_profile_or_stack_mode ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 132 filtered out; finished in 0.00s

```
```text
$ nix develop -c cargo test -p crunch-release-core stack_provenance --lib

running 8 tests
test manifest::tests::stack_provenance_policy_rejects_required_absent_evidence ... ok
test manifest::tests::stack_provenance_policy_accepts_optional_absent_evidence ... ok
test manifest::tests::stack_provenance_policy_accepts_optional_present_evidence ... ok
test manifest::tests::stack_provenance_policy_accepts_required_valid_evidence ... ok
test manifest::tests::validate_rejects_stack_provenance_digest_mismatch ... ok
test manifest::tests::validate_rejects_stack_provenance_semantic_promotion ... ok
test manifest::tests::validate_accepts_stack_provenance_release_evidence_with_matching_sidecars ... ok
test manifest::tests::stack_provenance_policy_rejects_required_invalid_fixture_matrix ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 126 filtered out; finished in 0.00s

```
```text
$ nix develop -c cargo test -p mantle --bin mantle release_verify_accepts_onix_stack_release_profile

running 1 test
test tests::release_verify_accepts_onix_stack_release_profile ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1268 filtered out; finished in 0.00s

```
```text
$ nix develop -c cargo test -p mantle --bin mantle stack_provenance

running 2 tests
test release_evidence::tests::create_release_bundle_rejects_stack_provenance_without_binary_selection_for_multiple_binaries ... ok
test release_evidence::tests::create_and_verify_release_bundle_records_stack_provenance_evidence ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1267 filtered out; finished in 0.12s

```
## cairn validate
```text
{
  "change_issues": [],
  "changes": 8,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 27,
  "valid": true
}
```
## cairn gate proposal
```text
{
  "change": "required-stack-provenance-profile",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8c93bca0e0ccc919150a4ca8794f39229c442c41ca196d5ce5fd02975078bbb5",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "940fb9e09605597177e41be36b7a6be10e1ddf27226c33653e74e001fc69a147",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## cairn gate design
```text
{
  "change": "required-stack-provenance-profile",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1efb563a2e51fa317f2f9c4bf9283dc659c2eea35bd491c13535302a708bf57a",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "91a331aceec384a1ad175438344706f8ee16c8e1cc59843d8adeb76c2ff36557",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## cairn gate tasks
```text
{
  "change": "required-stack-provenance-profile",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a75e309b0875f2d421db7519ff37bd9193e8e44ad9f26f70d7ea6148f64bb141",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "4935a72f1744ad22414684e6b1cac74b7b4628ff13edfc62c5a5aaa481517f31",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## accepted spec manual merge note
Cairn sync executed with reasons=[] but did not mutate the full-spec-shaped delta, so the stack-profile requirements were manually appended to cairn/specs/release-provenance/spec.md before archive.
## post-merge accepted requirement IDs
```text
88:r[mantle.release_provenance.stack_profile.required] Mantle MUST support a release profile for Onix stack artifacts that requires Valence stack-provenance sidecar and graph-report evidence while preserving optional stack provenance for generic releases.
91:r[mantle.release_provenance.stack_profile.positive]
98:r[mantle.release_provenance.stack_profile.required_absent]
105:r[mantle.release_provenance.stack_profile.constants] Mantle MUST derive or validate stack-provenance role, schema, claim-scope, and required non-claim constants from a reviewed source so CLI text, manifest validation, and documentation do not drift.
108:r[mantle.release_provenance.stack_profile.constants.match]
114:r[mantle.release_provenance.stack_profile.constants.drift]
120:r[mantle.release_provenance.stack_profile.negative] Mantle MUST include fail-closed fixtures for required stack-provenance profile failures.
123:r[mantle.release_provenance.stack_profile.negative.invalid]
129:r[mantle.release_provenance.stack_profile.docs] Mantle operator documentation MUST distinguish generic optional stack provenance from required Onix stack release profiles.
132:r[mantle.release_provenance.stack_profile.docs.generic]
138:r[mantle.release_provenance.stack_profile.docs.boundary]
144:r[mantle.release_provenance.stack_profile.validation] The change MUST include focused release-evidence tests, profile/constant checks, and Cairn validation evidence before archive.
147:r[mantle.release_provenance.stack_profile.validation.fixtures]
```
## post-merge cairn validate
```text
{
  "change_issues": [],
  "changes": 8,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 27,
  "valid": true
}
```
## post-merge cairn gate proposal
```text
{
  "change": "required-stack-provenance-profile",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8c93bca0e0ccc919150a4ca8794f39229c442c41ca196d5ce5fd02975078bbb5",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "940fb9e09605597177e41be36b7a6be10e1ddf27226c33653e74e001fc69a147",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate design
```text
{
  "change": "required-stack-provenance-profile",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1efb563a2e51fa317f2f9c4bf9283dc659c2eea35bd491c13535302a708bf57a",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "91a331aceec384a1ad175438344706f8ee16c8e1cc59843d8adeb76c2ff36557",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate tasks
```text
{
  "change": "required-stack-provenance-profile",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a75e309b0875f2d421db7519ff37bd9193e8e44ad9f26f70d7ea6148f64bb141",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "4935a72f1744ad22414684e6b1cac74b7b4628ff13edfc62c5a5aaa481517f31",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
```text
{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 26,
  "valid": true
}
```
## post-archive status
```text
## main...origin/main
 M README.md
 D cairn/changes/required-stack-provenance-profile/design.md
 D cairn/changes/required-stack-provenance-profile/proposal.md
 D cairn/changes/required-stack-provenance-profile/specs/release-provenance/spec.md
 D cairn/changes/required-stack-provenance-profile/tasks.md
 M cairn/specs/release-provenance/spec.md
 M crates/crunch-release-core/src/lib.rs
 M crates/crunch-release-core/src/manifest.rs
 M docs/operator-workflows.md
 M src/main.rs
 M src/release_cmd.rs
?? cairn/archive/2026-07-09-required-stack-provenance-profile/
?? cairn/changes/adopt-cap-std-release-boundaries/
?? cairn/changes/nix-evidence-core/
?? cairn/changes/oxide-release-worker-patterns/
?? cairn/changes/rustc-dev-guide-planning-boundaries/
```
