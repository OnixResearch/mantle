# Validation evidence: provider-fixed-point-global-eligibility

Date: 2026-07-03

## focused global release helper tests
```text
$ nix develop -c cargo test -p mantle --bin mantle global_reproducibility_release
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.63s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 5 tests
test global_reproducibility_release::tests::self_hosting_summary_parser_requires_strict_and_empty_fallbacks ... ok
test global_reproducibility_release::tests::release_derived_stage2_surface_evaluates_eligible ... ok
test global_reproducibility_release::tests::release_derived_full_release_admits_verified_provider_fixed_point_surface ... ok
test global_reproducibility_release::tests::release_derived_full_release_blocks_invalid_provider_fixed_point_surface ... ok
test global_reproducibility_release::tests::shell_writes_release_surface_evidence ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1095 filtered out; finished in 0.00s

```

## focused global evaluator shell tests
```text
$ nix develop -c cargo test -p mantle --bin mantle global_reproducibility_cmd
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 2 tests
test global_reproducibility_cmd::tests::shell_allows_missing_evidence_but_report_blocks_global_claim ... ok
test global_reproducibility_cmd::tests::shell_loads_evidence_and_writes_canonical_report ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1098 filtered out; finished in 0.01s

```

## provider fixed-point verifier tests
```text
$ nix develop -c cargo test -p mantle --bin mantle provider_fixed_point_verifier
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 5 tests
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_stage_digest_mismatch ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_accepts_valid_bounded_bundle ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_enforced_closure ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_bounded_non_claims ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rebases_copied_bundle_stage_paths ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1095 filtered out; finished in 0.01s

```

## full release evidence regeneration
```text
$ /home/brittonr/.cargo-target/debug/mantle --json release global-reproducibility-evidence --universe target/global-reproducibility/provider-bound-release-2026-07-02-full/universe.json --policy target/global-reproducibility/provider-bound-release-2026-07-02-full/policy.json --bundle-dir /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/release-evidence --verification-dir /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/release-verification --release-verify-json /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/final-release-verify.json --evidence-path target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json
{"evidence":[{"action_receipt_digest_blake3":"8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","fresh_rebuild_store":true,"hermeticity_evidence_digest_blake3":"d3b5e7fe8d146ee4d6aaaf0df27aeb9e59f658617ba724e644109ebdf93c24d3","output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","schema":"mantle-global-reproducibility-surface-evidence-v1","source_acquisition_digest_blake3":"692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","strict_hermeticity":true,"surface_id":"provider-bound-release-2026-07-02-01-mantle","toolchain_provenance_digest_blake3":"c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093","universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","witnesses":[{"host_class":"nixos-26.11-aspen1-unshare-fuse","identity":"aspen1-external-witness","operator_domain":"aspen1-external-witness","output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"perturbation_axes":[],"trust_status":"valid"}]},{"action_receipt_digest_blake3":"8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","fresh_rebuild_store":true,"hermeticity_evidence_digest_blake3":"6313b24f212232c09793de3b0e09db0e81e785ba3c5ceb86b6b106dd818840f1","output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","schema":"mantle-global-reproducibility-surface-evidence-v1","source_acquisition_digest_blake3":"692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","strict_hermeticity":true,"surface_id":"provider-bound-release-2026-07-02-stage2-mantle","toolchain_provenance_digest_blake3":"6df07d7a301ba14a332cc49d483558f9ab2524f5ccfc90043faffa27777eb9db","universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","witnesses":[{"host_class":"nixos-26.11-aspen1-unshare-fuse","identity":"aspen1-external-witness","operator_domain":"aspen1-external-witness","output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"perturbation_axes":[],"trust_status":"valid"}]}],"evidence_path":"/home/brittonr/git/mantle/target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json","kind":"mantle-global-reproducibility-release-surface-evidence-v1","policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","surface_count":2,"universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2"}
```

## full release global report regeneration
```text
$ /home/brittonr/.cargo-target/debug/mantle --json release global-reproducibility --universe target/global-reproducibility/provider-bound-release-2026-07-02-full/universe.json --policy target/global-reproducibility/provider-bound-release-2026-07-02-full/policy.json --evidence target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json --report-path target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release-report.json
{"kind":"mantle-global-reproducibility-evaluation-v1","report":{"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"claim_class":"eligible","evidence_digests_blake3":["6313b24f212232c09793de3b0e09db0e81e785ba3c5ceb86b6b106dd818840f1","692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","6df07d7a301ba14a332cc49d483558f9ab2524f5ccfc90043faffa27777eb9db","8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf","8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093","d3b5e7fe8d146ee4d6aaaf0df27aeb9e59f658617ba724e644109ebdf93c24d3","d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"included_surface_count":2,"non_claims":["excluded-surface:all-other-mantle-build-surfaces","not-compiler-correctness","not-deploy-success","not-future-code","not-physical-target-determinism","not-undeclared-frontends","not-undeclared-target-systems"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","policy_id":"single-aspen-external-witness-full-release-strict","schema":"mantle-global-reproducibility-report-v1","surfaces":[{"accepted_host_classes":["nixos-26.11-aspen1-unshare-fuse"],"accepted_operator_domains":["aspen1-external-witness"],"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"cache_substitution_mode":"release-proof-recorded-no-global-cache-claim","covered_perturbation_axes":[],"non_claims":["not-compiler-correctness","not-deploy-success","not-future-code"],"output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"release_artifact_set":"binaries/01-mantle","source_acquisition_mode":"public-release-source-archive","surface_id":"provider-bound-release-2026-07-02-01-mantle","target_system":"x86_64-linux","toolchain_route":"provider-fixed-point-source-built-handoff","witness_counts":{"accepted":1,"failed_digest_mismatched":0,"failed_invalid_signature":0,"policy_insufficient":0,"skipped_revoked":0,"skipped_same_domain":0,"skipped_unknown_key":0}},{"accepted_host_classes":["nixos-26.11-aspen1-unshare-fuse"],"accepted_operator_domains":["aspen1-external-witness"],"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"cache_substitution_mode":"release-proof-recorded-no-global-cache-claim","covered_perturbation_axes":[],"non_claims":["not-compiler-correctness","not-deploy-success","not-future-code"],"output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"release_artifact_set":"binaries/02-stage2-mantle","source_acquisition_mode":"public-release-source-archive","surface_id":"provider-bound-release-2026-07-02-stage2-mantle","target_system":"x86_64-linux","toolchain_route":"mantle-self-hosting-proof-v2-stage2-strict-with-aspen-witness","witness_counts":{"accepted":1,"failed_digest_mismatched":0,"failed_invalid_signature":0,"policy_insufficient":0,"skipped_revoked":0,"skipped_same_domain":0,"skipped_unknown_key":0}}],"universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","universe_name":"provider-bound-release-2026-07-02 full release artifacts","witness_policy":"one-valid-external-domain-and-host-class"},"report_digest_blake3":"ddcf15edea2b0b2e10ef653fcf1efe89803bfa6db49ee3c4ab7ee62ceb6e7b8f","report_path":"/home/brittonr/git/mantle/target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release-report.json"}
```

## generated artifact digests
```text
$ nix run nixpkgs#b3sum -- target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.stdout.json target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.stderr.txt target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release-report.json target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release.stdout.json target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release.stderr.txt
06b99a0968cbe4afb705b42318fdb3f16a01a3061b45c1cee3962175828c14fb  target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json
61377b928bce801ec2f71006f90005e469e47c6982e69af500c083c9782b116d  target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.stdout.json
af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262  target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.stderr.txt
ddcf15edea2b0b2e10ef653fcf1efe89803bfa6db49ee3c4ab7ee62ceb6e7b8f  target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release-report.json
8a52c4a4f3e1899e4ce9c29a349a74bbbd0f1db4a9e4ef2c19b928f1ec6ee6d5  target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release.stdout.json
af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262  target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release.stderr.txt
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
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal provider-fixed-point-global-eligibility --root .
{
  "change": "provider-fixed-point-global-eligibility",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "007d2bae890db947b06249214db42fe9a66f2d59c6b272b570606d4f4f54aaa2",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "3488c3a088fd76b10e962181a213903c79a68622b39de6d49d83628e99083587",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design provider-fixed-point-global-eligibility --root .
{
  "change": "provider-fixed-point-global-eligibility",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "593578fdef1125683fda1e90a26c52d61663af6c301eddb476ad0cce81288b1d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "9dfb81d422e86b5d4a4215f314f96394dc18de5469bfc7907dfac500d97c8b27",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks provider-fixed-point-global-eligibility --root .
{
  "change": "provider-fixed-point-global-eligibility",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b74f1d3eeb7591a36ef79da487e08737cb6755fafa3e345590159ee1f0cb096c",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "aff89486c2829716807ded9880f5ba9b2bd56a62854fdf136ad1d468e0c2b1ad",
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
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks provider-fixed-point-global-eligibility --root .
{
  "change": "provider-fixed-point-global-eligibility",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "2fc4b254c62de55c427568426095099942236ac704322cd16e3eca379c38676d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0047933092bff678cd6ea105b2f558d33db669e3d4c0e3ee2ef26944e16870af",
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

## post-cleanup validation
```text
$ nix develop -c cargo test -p mantle --bin mantle global_reproducibility_release
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on build directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.12s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 5 tests
test global_reproducibility_release::tests::self_hosting_summary_parser_requires_strict_and_empty_fallbacks ... ok
test global_reproducibility_release::tests::release_derived_stage2_surface_evaluates_eligible ... ok
test global_reproducibility_release::tests::release_derived_full_release_admits_verified_provider_fixed_point_surface ... ok
test global_reproducibility_release::tests::release_derived_full_release_blocks_invalid_provider_fixed_point_surface ... ok
test global_reproducibility_release::tests::shell_writes_release_surface_evidence ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1095 filtered out; finished in 0.01s

$ /home/brittonr/.cargo-target/debug/mantle --json release global-reproducibility-evidence ...
{"evidence":[{"action_receipt_digest_blake3":"8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","fresh_rebuild_store":true,"hermeticity_evidence_digest_blake3":"d3b5e7fe8d146ee4d6aaaf0df27aeb9e59f658617ba724e644109ebdf93c24d3","output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","schema":"mantle-global-reproducibility-surface-evidence-v1","source_acquisition_digest_blake3":"692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","strict_hermeticity":true,"surface_id":"provider-bound-release-2026-07-02-01-mantle","toolchain_provenance_digest_blake3":"c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093","universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","witnesses":[{"host_class":"nixos-26.11-aspen1-unshare-fuse","identity":"aspen1-external-witness","operator_domain":"aspen1-external-witness","output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"perturbation_axes":[],"trust_status":"valid"}]},{"action_receipt_digest_blake3":"8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","fresh_rebuild_store":true,"hermeticity_evidence_digest_blake3":"6313b24f212232c09793de3b0e09db0e81e785ba3c5ceb86b6b106dd818840f1","output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","schema":"mantle-global-reproducibility-surface-evidence-v1","source_acquisition_digest_blake3":"692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","strict_hermeticity":true,"surface_id":"provider-bound-release-2026-07-02-stage2-mantle","toolchain_provenance_digest_blake3":"6df07d7a301ba14a332cc49d483558f9ab2524f5ccfc90043faffa27777eb9db","universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","witnesses":[{"host_class":"nixos-26.11-aspen1-unshare-fuse","identity":"aspen1-external-witness","operator_domain":"aspen1-external-witness","output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"perturbation_axes":[],"trust_status":"valid"}]}],"evidence_path":"/home/brittonr/git/mantle/target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json","kind":"mantle-global-reproducibility-release-surface-evidence-v1","policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","surface_count":2,"universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2"}
$ /home/brittonr/.cargo-target/debug/mantle --json release global-reproducibility ...
{"kind":"mantle-global-reproducibility-evaluation-v1","report":{"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"claim_class":"eligible","evidence_digests_blake3":["6313b24f212232c09793de3b0e09db0e81e785ba3c5ceb86b6b106dd818840f1","692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba","6df07d7a301ba14a332cc49d483558f9ab2524f5ccfc90043faffa27777eb9db","8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf","8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260","c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093","d3b5e7fe8d146ee4d6aaaf0df27aeb9e59f658617ba724e644109ebdf93c24d3","d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"included_surface_count":2,"non_claims":["excluded-surface:all-other-mantle-build-surfaces","not-compiler-correctness","not-deploy-success","not-future-code","not-physical-target-determinism","not-undeclared-frontends","not-undeclared-target-systems"],"policy_digest_blake3":"03da83116b1432a520bcc7c07e0213d486615d4bd5baa83476374a7c304eb4d8","policy_id":"single-aspen-external-witness-full-release-strict","schema":"mantle-global-reproducibility-report-v1","surfaces":[{"accepted_host_classes":["nixos-26.11-aspen1-unshare-fuse"],"accepted_operator_domains":["aspen1-external-witness"],"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"cache_substitution_mode":"release-proof-recorded-no-global-cache-claim","covered_perturbation_axes":[],"non_claims":["not-compiler-correctness","not-deploy-success","not-future-code"],"output_digest_set_blake3":["8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf"],"release_artifact_set":"binaries/01-mantle","source_acquisition_mode":"public-release-source-archive","surface_id":"provider-bound-release-2026-07-02-01-mantle","target_system":"x86_64-linux","toolchain_route":"provider-fixed-point-source-built-handoff","witness_counts":{"accepted":1,"failed_digest_mismatched":0,"failed_invalid_signature":0,"policy_insufficient":0,"skipped_revoked":0,"skipped_same_domain":0,"skipped_unknown_key":0}},{"accepted_host_classes":["nixos-26.11-aspen1-unshare-fuse"],"accepted_operator_domains":["aspen1-external-witness"],"accepted_witness_identities":["aspen1-external-witness"],"blockers":[],"cache_substitution_mode":"release-proof-recorded-no-global-cache-claim","covered_perturbation_axes":[],"non_claims":["not-compiler-correctness","not-deploy-success","not-future-code"],"output_digest_set_blake3":["d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71"],"release_artifact_set":"binaries/02-stage2-mantle","source_acquisition_mode":"public-release-source-archive","surface_id":"provider-bound-release-2026-07-02-stage2-mantle","target_system":"x86_64-linux","toolchain_route":"mantle-self-hosting-proof-v2-stage2-strict-with-aspen-witness","witness_counts":{"accepted":1,"failed_digest_mismatched":0,"failed_invalid_signature":0,"policy_insufficient":0,"skipped_revoked":0,"skipped_same_domain":0,"skipped_unknown_key":0}}],"universe_digest_blake3":"ab5cb6a4fe4f411b5b4cc55e43f85001dccb9bcc7ec5aa6496b0d95f6bc085a2","universe_name":"provider-bound-release-2026-07-02 full release artifacts","witness_policy":"one-valid-external-domain-and-host-class"},"report_digest_blake3":"ddcf15edea2b0b2e10ef653fcf1efe89803bfa6db49ee3c4ab7ee62ceb6e7b8f","report_path":"/home/brittonr/git/mantle/target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release-report.json"}
$ nix run nixpkgs#b3sum -- target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release-report.json
06b99a0968cbe4afb705b42318fdb3f16a01a3061b45c1cee3962175828c14fb  target/global-reproducibility/provider-bound-release-2026-07-02-full/evidence.json
ddcf15edea2b0b2e10ef653fcf1efe89803bfa6db49ee3c4ab7ee62ceb6e7b8f  target/global-reproducibility/provider-bound-release-2026-07-02-full/eligible-full-release-report.json
$ nix develop -c cargo fmt --check -p mantle
$ git diff --check
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
