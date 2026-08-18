# Validation — global reproducibility release evidence

Date: 2026-07-03

## cargo fmt
```text
$ nix develop -c cargo fmt --check -p mantle
exit_status=0
```

## global reproducibility focused tests
```text
$ nix develop -c cargo test -p mantle --bin mantle global_reproducibility
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 7 tests
test global_reproducibility_release::tests::self_hosting_summary_parser_requires_strict_and_empty_fallbacks ... ok
test release_cmd::tests::release_verify_json_keeps_global_reproducibility_non_global ... ok
test global_reproducibility_release::tests::release_derived_stage2_surface_evaluates_eligible ... ok
test global_reproducibility_release::tests::release_derived_full_release_blocks_provider_fixed_point_surface ... ok
test global_reproducibility_cmd::tests::shell_loads_evidence_and_writes_canonical_report ... ok
test global_reproducibility_cmd::tests::shell_allows_missing_evidence_but_report_blocks_global_claim ... ok
test global_reproducibility_release::tests::shell_writes_release_surface_evidence ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1091 filtered out; finished in 0.01s

exit_status=0
```

## full release helper evidence command
```text
$ /home/brittonr/.cargo-target/debug/mantle --json release global-reproducibility-evidence ...
{"evidence":[{"action_receipt_digest_blake3":"8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","fresh_rebuild_store":false,"hermeticity_evidence_digest_blake3":"a085615b538a8a588a6d388f862605e4d06e1dc8671dd26d2670020630eee8cc","output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","schema":"mantle-global-reproducibility-surface-evidence-v1","source_acquisition_digest_blake3":"692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","strict_hermeticity":false,"surface_id":"provider-bound-release-2026-07-02-01-mantle","toolchain_provenance_digest_blake3":"a085615b538a8a588a6d388f862605e4d06e1dc8671dd26d2670020630eee8cc","universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","unsupported_reason":"provider fixed-point handoff evidence is release-bounded but is not admitted as strict/fresh global reproducibility evidence","witnesses":[{"host_class":"nixos-26.11-aspen1-unshare-fuse","identity":"aspen1-external-witness","operator_domain":"aspen1-external-witness","output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"perturbation_axes":[],"trust_status":"valid"}]},{"action_receipt_digest_blake3":"8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","fresh_rebuild_store":true,"hermeticity_evidence_digest_blake3":"6313b24f212232c09793de3b0e09db0e81e785ba3c5ceb86b6b106dd818840f1","output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","schema":"mantle-global-reproducibility-surface-evidence-v1","source_acquisition_digest_blake3":"692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","strict_hermeticity":true,"surface_id":"provider-bound-release-2026-07-02-stage2-mantle","toolchain_provenance_digest_blake3":"6df07d7a301ba14a332cc49d483558f9ab2524f5ccfc90043faffa27777eb9db","universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","witnesses":[{"host_class":"nixos-26.11-aspen1-unshare-fuse","identity":"aspen1-external-witness","operator_domain":"aspen1-external-witness","output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"perturbation_axes":[],"trust_status":"valid"}]}],"evidence_path":"/home/brittonr/git/mantle/target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json","kind":"mantle-global-reproducibility-release-surface-evidence-v1","policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","surface_count":2,"universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2"}
exit_status=0
```

## full release global report command
```text
$ /home/brittonr/.cargo-target/debug/mantle --json release global-reproducibility ...
{"kind":"mantle-global-reproducibility-evaluation-v1","report":{"accepted_witness_identities":["aspen1-external-witness"],"blockers":[{"evidence_class":"reused-store","message":"surface evidence reused a store where fresh replay was required","next_action":"rerun independent replay with a fresh store identity","surface_id":"provider-bound-release-2026-07-02-01-mantle"},{"evidence_class":"unsupported-surface","message":"provider fixed-point handoff evidence is release-bounded but is not admitted as strict/fresh global reproducibility evidence","next_action":"remove the surface from the included universe or add supported evidence for it","surface_id":"provider-bound-release-2026-07-02-01-mantle"},{"evidence_class":"weak-hermeticity","message":"surface evidence is not strict-hermetic","next_action":"rerun the surface under strict hermeticity and record the resulting receipt","surface_id":"provider-bound-release-2026-07-02-01-mantle"}],"claim_class":"blocked","evidence_digests_blake3":["6313b24f212232c09793de3b0e09db0e81e785ba3c5ceb86b6b106dd818840f1","692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","6df07d7a301ba14a332cc49d483558f9ab2524f5ccfc90043faffa27777eb9db","8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf","8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","a085615b538a8a588a6d388f862605e4d06e1dc8671dd26d2670020630eee8cc","d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"included_surface_count":2,"non_claims":["excluded-surface:all-other-mantle-build-surfaces","global-reproducibility-blocked","not-compiler-correctness","not-deploy-success","not-future-code","not-physical-target-determinism","not-undeclared-frontends","not-undeclared-target-systems"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","policy_id":"single-aspen-external-witness-full-release-strict","schema":"mantle-global-reproducibility-report-v1","surfaces":[{"accepted_host_classes":["nixos-26.11-aspen1-unshare-fuse"],"accepted_operator_domains":["aspen1-external-witness"],"accepted_witness_identities":["aspen1-external-witness"],"blockers":[{"evidence_class":"reused-store","message":"surface evidence reused a store where fresh replay was required","next_action":"rerun independent replay with a fresh store identity","surface_id":"provider-bound-release-2026-07-02-01-mantle"},{"evidence_class":"unsupported-surface","message":"provider fixed-point handoff evidence is release-bounded but is not admitted as strict/fresh global reproducibility evidence","next_action":"remove the surface from the included universe or add supported evidence for it","surface_id":"provider-bound-release-2026-07-02-01-mantle"},{"evidence_class":"weak-hermeticity","message":"surface evidence is not strict-hermetic","next_action":"rerun the surface under strict hermeticity and record the resulting receipt","surface_id":"provider-bound-release-2026-07-02-01-mantle"}],"cache_substitution_mode":"release-proof-recorded-no-global-cache-claim","covered_perturbation_axes":[],"non_claims":["not-compiler-correctness","not-deploy-success","not-future-code"],"output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"release_artifact_set":"binaries/01-mantle","source_acquisition_mode":"public-release-source-archive","surface_id":"provider-bound-release-2026-07-02-01-mantle","target_system":"x86_64-linux","toolchain_route":"provider-fixed-point-source-built-handoff","witness_counts":{"accepted":1,"failed_digest_mismatched":0,"failed_invalid_signature":0,"policy_insufficient":0,"skipped_revoked":0,"skipped_same_domain":0,"skipped_unknown_key":0}},{"accepted_host_classes":["nixos-26.11-aspen1-unshare-fuse"],"accepted_operator_domains":["aspen1-external-witness"],"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"cache_substitution_mode":"release-proof-recorded-no-global-cache-claim","covered_perturbation_axes":[],"non_claims":["not-compiler-correctness","not-deploy-success","not-future-code"],"output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"release_artifact_set":"binaries/02-stage2-mantle","source_acquisition_mode":"public-release-source-archive","surface_id":"provider-bound-release-2026-07-02-stage2-mantle","target_system":"x86_64-linux","toolchain_route":"mantle-self-hosting-proof-v2-stage2-strict-with-aspen-witness","witness_counts":{"accepted":1,"failed_digest_mismatched":0,"failed_invalid_signature":0,"policy_insufficient":0,"skipped_revoked":0,"skipped_same_domain":0,"skipped_unknown_key":0}}],"universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","universe_name":"provider-bound-release-2026-07-02 full release artifacts","witness_policy":"one-valid-external-domain-and-host-class"},"report_digest_blake3":"f6727befe87856b939d3de6f1771ab75b672e612cc978a0c7f3d9db147890467","report_path":"/home/brittonr/git/mantle/target/global-reproducibility/provider-bound-release-2026-07-02-full/blocked-full-release-report.json"}
{"error":"global reproducibility claim blocked: 3 blocker(s)","code":3,"kind":"internal"}
exit_status=3 (expected blocked full release universe)
```

## stage2 helper report command
```text
$ /home/brittonr/.cargo-target/debug/mantle --json release global-reproducibility ... stage2 helper evidence
{"kind":"mantle-global-reproducibility-evaluation-v1","report":{"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"claim_class":"eligible","evidence_digests_blake3":["6313b24f212232c09793de3b0e09db0e81e785ba3c5ceb86b6b106dd818840f1","692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","6df07d7a301ba14a332cc49d483558f9ab2524f5ccfc90043faffa27777eb9db","8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"included_surface_count":1,"non_claims":["excluded-surface:all-other-mantle-build-surfaces","excluded-surface:provider-bound-release-2026-07-02-01-mantle","not-compiler-correctness","not-deploy-success","not-future-code","not-physical-target-determinism","not-undeclared-frontends","not-undeclared-target-systems"],"policy_digest_blake3":"aa35734ed3edfeb13e8c7fab583fef9860c3f306d8c5e05fd40f4a65c155f1e3","policy_id":"single-aspen-external-witness-stage2-strict","schema":"mantle-global-reproducibility-report-v1","surfaces":[{"accepted_host_classes":["nixos-26.11-aspen1-unshare-fuse"],"accepted_operator_domains":["aspen1-external-witness"],"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"cache_substitution_mode":"release-proof-recorded-no-global-cache-claim","covered_perturbation_axes":[],"non_claims":["not-compiler-correctness","not-deploy-success","not-future-code"],"output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"release_artifact_set":"binaries/02-stage2-mantle","source_acquisition_mode":"public-release-source-archive","surface_id":"provider-bound-release-2026-07-02-stage2-mantle","target_system":"x86_64-linux","toolchain_route":"mantle-self-hosting-proof-v2-stage2-strict-with-aspen-witness","witness_counts":{"accepted":1,"failed_digest_mismatched":0,"failed_invalid_signature":0,"policy_insufficient":0,"skipped_revoked":0,"skipped_same_domain":0,"skipped_unknown_key":0}}],"universe_digest_blake3":"8d7f292084fd41051f4db9587872dc78f627617acfcd9b5a35c3c866c982ac33","universe_name":"provider-bound-release-2026-07-02 strict stage2 artifact","witness_policy":"one-valid-external-domain-and-host-class"},"report_digest_blake3":"255f9caf8dbd420e9e78093b7d0aee3c5914def80d2f9debdb7b9e45353b710a","report_path":"/home/brittonr/git/mantle/target/global-reproducibility/provider-bound-release-2026-07-02-stage2/helper-eligible-report.json"}
exit_status=0
```

## git diff check
```text
$ git diff --check
exit_status=0
```

## cairn validate
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
exit_status=0
```

## cairn gates
```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal global-reproducibility-release-evidence --root .
{
  "change": "global-reproducibility-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e3640d1a291c7bf4e7e1ef440b692bd1e5232c3d74d6eba75a78400614a36a90",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e3a89f53978383abc62b90c1854af9bef05a83340a052f14fa99dde5ffa11b49",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design global-reproducibility-release-evidence --root .
{
  "change": "global-reproducibility-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a5568752461f395da67420b0b8e28d1b98d76b655e7c7e78464b8fc5d62c507e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "392cc36f69ce8441ac9986e1dfa4c15d6cf2cf911488dcd25d24dc1ade2ef58c",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks global-reproducibility-release-evidence --root .
{
  "change": "global-reproducibility-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8eb98353acaee68ff6727185b7b5e098cc0524ad5153fc349505363ea7864a4a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1bd4167f4d877b1d583dc8620440f05b9f9fbfdcaa2a157810ca71c205928490",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```

## post-task-update cairn validate and gates
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
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal global-reproducibility-release-evidence --root .
{
  "change": "global-reproducibility-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "9233f4d6d63dfeb6eb87911890f6f9066590de6b0a283b74a4bee8c7dcca1cb2",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1634684e49bc2bdc093414a17ac1a2dba295b126cb6c750ee9ca639e2df7ed3f",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design global-reproducibility-release-evidence --root .
{
  "change": "global-reproducibility-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "3c5ac8d1a08b6ad2d757a594a710719b9a39188358cab05c57c28c989cc59fb9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b828683e6f4f87baecaa8052212625c971c152bc3a70360a111f68090a63e64e",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks global-reproducibility-release-evidence --root .
{
  "change": "global-reproducibility-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f863e0b5bdc1e9c4bb53b6d8180aa9ca7103527eea81264308e12d485f0494b0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "228f88cd6c07645887efd67b7ee2be9b9f95ac04c2c493feeaa1f11aa188dcc8",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
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
$ nix run path:/home/brittonr/git/cairn#cairn -- change list --root .
{
  "changes": [],
  "layout": "cairn",
  "root": "."
}
exit_status=0
```

## post-manual-spec-sync validation
```text
$ rg verification_evidence.global_reproducibility_release_surface_evidence cairn/specs/verification-evidence/spec.md tools/tracey_refs.rs
tools/tracey_refs.rs:// r[impl verification_evidence.global_reproducibility_release_surface_evidence]
tools/tracey_refs.rs:// r[verify verification_evidence.global_reproducibility_release_surface_evidence]
cairn/specs/verification-evidence/spec.md:r[verification_evidence.global_reproducibility_release_surface_evidence] Mantle MUST provide deterministic release-bundle-derived surface evidence for global reproducibility reports without weakening the global admission gate.
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
$ git diff --check
exit_status=0
```
