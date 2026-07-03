# External release witness handoff success

Task-ID: external-release-witness-handoff
Covers: r[release_witness.external_handoff]
Date: 2026-07-02

## Scope

The earlier `provider-bound-release-evidence-2026-06-28-provider-remap-fixed` request remains blocker evidence because its original proof staged `vendor/.pi/prompt-history.jsonl`, which was absent from the public source archive. This transcript records the replacement release evidence bundle created after fixing the staging/archive policy mismatch.

Fresh release ID: `provider-bound-release-evidence-2026-07-02-source-policy-fixed`

Generated artifact root, intentionally outside the repo:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed
```

## Code fix evidence

The implementation now uses the release source archive policy when staging self-build source paths and keeps `target/` as a root-only archive exclusion so vendored source files such as `vendor-deps/cc/src/target/apple.rs` are preserved.

Validation commands run before the fresh proof:

```text
rustfmt --check src/self_build.rs crates/crunch-release-core/src/source_archive.rs
cargo test -p crunch-release-core source_archive -- --nocapture
cargo test -p mantle --bin mantle copy_selected_source_tree_copies_only_allowlisted_entries -- --nocapture
cargo test -p mantle --bin mantle staged_source_path_policy -- --nocapture
cargo test -p mantle --bin mantle self_build::tests:: -- --nocapture
cargo test -p crunch-release-core -- --nocapture
```

Observed results:

```text
self_build::tests::copy_selected_source_tree_copies_only_allowlisted_entries ... ok
self_build::tests::staged_source_path_policy_rejects_unsafe_relative_paths ... ok
self_build::tests::staged_source_path_policy_matches_release_archive_policy ... ok
source_archive::tests::source_paths_with_target_named_components_are_preserved ... ok
cargo test -p mantle --bin mantle self_build::tests:: ... 157 passed; 0 failed
cargo test -p crunch-release-core ... 70 passed; 0 failed
```

Cairn/whitespace validation before proof packaging:

```text
git diff --check
/home/brittonr/.cargo-target/debug/cairn validate
/home/brittonr/.cargo-target/debug/cairn gate proposal external-release-witness-handoff
/home/brittonr/.cargo-target/debug/cairn gate design external-release-witness-handoff
/home/brittonr/.cargo-target/debug/cairn gate tasks external-release-witness-handoff
```

All gates returned `valid: true` / `verdict: PASS`.

## Fresh release evidence

Full self-hosting proof bundle:

```text
/home/brittonr/git/mantle/target/self-hosting-proof/release-source-policy-fixed-2026-07-02-rerun2
```

Proof result:

```text
test self_hosting_stage0_stage1_stage2 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 2345.12s
stage1_binary: d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71 binaries/stage1-mantle
stage2_binary: d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71 binaries/stage2-mantle
stage1_equals_stage2: true
stage2_hermeticity_mode: strict
stage2_fallback_events: []
```

Provider fixed-point proof bundle:

```text
/home/brittonr/.cargo-target/repo-targets/mantle/provider-fixed-point-release-source-policy-fixed-2026-07-02
```

Provider proof result:

```text
Cargo-free fixed-point: success
stage1_binary_blake3: 8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf
stage2_binary_blake3: 8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf
```

Release bundle verification with bundled provider proof:

```text
release evidence bundle: /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/release-evidence
release id: provider-bound-release-evidence-2026-07-02-source-policy-fixed
source archive: source/.tmpeqlrwy
proof bundle: proof/self-hosting
binaries: 2
provider fixed-point proof: proof/provider-fixed-point
manifest: manifest.json
```

Key release verification fields:

```text
binaries/01-mantle:        8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf
binaries/02-stage2-mantle: d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71
source archive digest:     692a25feed506ebf83d20f85003de03a607c9b7655952d3562687117c23008ba
provider proof digest:     a085615b538a8a588a6d388f862605e4d06e1dc8671dd26d2670020630eee8cc
provider proof status:     valid
provider matched artifact: binaries/01-mantle
```

Source archive policy check:

```text
private .pi paths:
vendored target component sample:
vendor-deps/cc/src/target/apple.rs
root target paths:
```

## Public witness request

Request directory:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/external-witness-request-2026-07-02
```

Request metadata:

```json
{"schema":"mantle-witness-request-v1","request_layout_version":1,"release_id":"provider-bound-release-evidence-2026-07-02-source-policy-fixed","release_bundle_relative_path":"release-evidence/provider-bound-release-evidence-2026-07-02-source-policy-fixed","verification_seed_relative_path":"release-verification/provider-bound-release-evidence-2026-07-02-source-policy-fixed"}
```

Canonical request-directory digest:

```text
1f5f4f527bcd0499b98682f8f62189a7c341477f58d946aab865876d4a30eeb5  -
```

Private-state check:

```text
public request private-state check: no policy.json, revocations.json, *.key, signing-key*, or *secret* files present
```

Release trusted public key:

```text
crunch-britton-desktop-1:o0kC+fsdJtKm8By36M/PD+OPspDxrbFytww7aniHHPg=
```

## Aspen external witness

Aspen verified the copied request digest:

```text
1f5f4f527bcd0499b98682f8f62189a7c341477f58d946aab865876d4a30eeb5  -
```

Aspen rebuild command used:

```text
/run/current-system/sw/bin/unshare --user --map-root-user --mount --fork \
  /home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/rebuild-witness-request.sh \
  --scratch-dir /home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/work-unshare-fuse-full \
  /home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/request \
  --identity aspen1-external-witness \
  --system x86_64-linux \
  --toolchain nightly-2025-11-01-source-policy-fixed \
  --host-class nixos-26.11-aspen1-unshare-fuse
```

Provider-bound replay inputs on Aspen:

```text
CRUNCH_WITNESS_RUST_SOURCE_PROVIDER=/home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out
CRUNCH_WITNESS_TOOLCHAIN_CLOSURE=/home/brittonr/git/mantle/target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json
CRUNCH_WITNESS_RUSTC=/home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/bin/rustc
CRUNCH_WITNESS_PROVIDER_TARGET=x86_64-unknown-linux-musl
CRUNCH_WITNESS_REBUILD_CLI_BIN=/home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/bin/mantle
CRUNCH_PROOF_RUSTUP_TOOLCHAIN=nightly-2025-11-01
```

Aspen status:

```text
exit_status=0
stdout=/home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/witness-rebuild-unshare-fuse-full.json
stderr=/home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/witness-rebuild-unshare-fuse-full.stderr
```

Aspen output:

```text
witness rebuild completed: /home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/request
release id: provider-bound-release-evidence-2026-07-02-source-policy-fixed
verification output: /home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/work-unshare-fuse-full/release-verification/provider-bound-release-evidence-2026-07-02-source-policy-fixed
witness attestation: .../witnesses/aspen1-external-witness.json
signature: .../witnesses/aspen1-external-witness.json.sig
rebuild audit: /home/brittonr/release-witness/mantle-aspen1-2026-07-02-source-policy-fixed/work-unshare-fuse-full/witness-rebuild-audit/meta.json
```

Aspen trusted public key:

```text
crunch-aspen1-1:eS7XaGA4DLwH1lRgOQ6x3Z1JBGLZDTYSh1TwU1BiNqg=
```

Returned sidecars imported from:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/external-witness-aspen1-2026-07-02/returned-sidecars/witnesses/aspen1-external-witness.json
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/external-witness-aspen1-2026-07-02/returned-sidecars/witnesses/aspen1-external-witness.json.sig
```

Import result:

```text
verification dir: /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/release-verification
imported witness identities: aspen1-external-witness
skipped exact duplicates: (none)
```

## Final verification

Final verification command used both trusted keys:

```text
mantle attest release-verify \
  /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/release-verification \
  --trusted-public-key crunch-britton-desktop-1:o0kC+fsdJtKm8By36M/PD+OPspDxrbFytww7aniHHPg= \
  --trusted-public-key crunch-aspen1-1:eS7XaGA4DLwH1lRgOQ6x3Z1JBGLZDTYSh1TwU1BiNqg=
```

Final verification result:

```json
{
  "release_attestation_digest": "7557adc8ea544fb15193ed8cec18804a149a4df3e22597e2ad46801604a45d84",
  "release_signer_key_name": "crunch-britton-desktop-1",
  "discovered_witness_count": 1,
  "considered_witness_count": 1,
  "technical_class": "external-witness-match",
  "policy_status": "satisfied",
  "final_class": "quorum-satisfied",
  "matching_witness_count": 1,
  "independent_witness_identities": 1,
  "revoked_witness_count": 0,
  "independent_agreement_status": "satisfied",
  "independent_agreement_class": "independent-rebuild-agreement",
  "independent_agreement_report_digest": "ef889e11a9a37978255344e43635f0f633ab9f268edff8170cf120157016e6df",
  "independent_agreement_counted_witness_count": 1,
  "independent_agreement_skipped_witness_count": 0,
  "independent_agreement_failed_witness_count": 0,
  "independent_agreement_witnesses": [
    {
      "witness_identity": "aspen1-external-witness",
      "signer_key_name": "crunch-aspen1-1",
      "signature_valid": true,
      "digest_match": true,
      "independence_domain": "aspen1-external-witness",
      "policy_counted": true,
      "classification_reason": "counted"
    }
  ]
}
```

## Negative fail-closed check

A witness import attempted with the Aspen witness JSON but without the `.sig` sidecar failed closed:

```text
exit_status=3
error: missing witness signature sidecar /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/negative-missing-signature/source/witnesses/aspen1-external-witness.json.sig: No such file or directory (os error 2)
```

## Final active-change validation

Final validation after tracked evidence and release-note edits:

```text
## final release verify
{
  "release_attestation_digest": "7557adc8ea544fb15193ed8cec18804a149a4df3e22597e2ad46801604a45d84",
  "release_signer_key_name": "crunch-britton-desktop-1",
  "discovered_witness_count": 1,
  "considered_witness_count": 1,
  "technical_class": "external-witness-match",
  "policy_status": "satisfied",
  "final_class": "quorum-satisfied",
  "matching_witness_count": 1,
  "independent_witness_identities": 1,
  "revoked_witness_count": 0,
  "independent_agreement_status": "satisfied",
  "independent_agreement_class": "independent-rebuild-agreement",
  "independent_agreement_report_digest": "ef889e11a9a37978255344e43635f0f633ab9f268edff8170cf120157016e6df",
  "independent_agreement_counted_witness_count": 1,
  "independent_agreement_skipped_witness_count": 0,
  "independent_agreement_failed_witness_count": 0,
  "independent_agreement_witnesses": [
    {
      "witness_identity": "aspen1-external-witness",
      "signer_key_name": "crunch-aspen1-1",
      "signature_valid": true,
      "digest_match": true,
      "independence_domain": "aspen1-external-witness",
      "policy_counted": true,
      "classification_reason": "counted"
    }
  ]
}
## git diff --check
## cairn validate
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
## cairn gate proposal
{
  "change": "external-release-witness-handoff",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
## cairn gate design
{
  "change": "external-release-witness-handoff",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
## cairn gate tasks
{
  "change": "external-release-witness-handoff",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Final validation rerun after marking V3 complete

```text
## final release verify
{
  "release_attestation_digest": "7557adc8ea544fb15193ed8cec18804a149a4df3e22597e2ad46801604a45d84",
  "release_signer_key_name": "crunch-britton-desktop-1",
  "discovered_witness_count": 1,
  "considered_witness_count": 1,
  "technical_class": "external-witness-match",
  "policy_status": "satisfied",
  "final_class": "quorum-satisfied",
  "matching_witness_count": 1,
  "independent_witness_identities": 1,
  "revoked_witness_count": 0,
  "independent_agreement_status": "satisfied",
  "independent_agreement_class": "independent-rebuild-agreement",
  "independent_agreement_report_digest": "ef889e11a9a37978255344e43635f0f633ab9f268edff8170cf120157016e6df",
  "independent_agreement_counted_witness_count": 1,
  "independent_agreement_skipped_witness_count": 0,
  "independent_agreement_failed_witness_count": 0,
  "independent_agreement_witnesses": [
    {
      "witness_identity": "aspen1-external-witness",
      "signer_key_name": "crunch-aspen1-1",
      "signature_valid": true,
      "digest_match": true,
      "independence_domain": "aspen1-external-witness",
      "policy_counted": true,
      "classification_reason": "counted"
    }
  ]
}
## git diff --check
## cairn validate
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
## cairn gate proposal
{
  "change": "external-release-witness-handoff",
  "input_hash": "ef08f6670fe2d9a7b3f1fe02f4e6332d14b218863fc37b6abb4e34e413ecfd6f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "receipt_hash": "1fadcf1edc210b675a5faea6d82a9971c7e21db29a3af0eda669fdb7de82d3cf",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
## cairn gate design
{
  "change": "external-release-witness-handoff",
  "input_hash": "edfc0fcd27bf0e51f932e24336f3d381a3e2a27c20abadd4d323083a4c0bf6a2",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "receipt_hash": "5537e3edabff2d97439b2d222197dd7b05cfa39bf6ce3961641cd2c50c81544f",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
## cairn gate tasks
{
  "change": "external-release-witness-handoff",
  "input_hash": "2240b051474e336ee70dad2171d1f9f947b5564511cdf8a7e8225d3ff3125278",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "receipt_hash": "92137c4d80f07a45c3ded8354473ce0a7d32935d51fc139bdcb8757782117d74",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Evidence hashes

```text
b83f6c519cfc139c583cf73156d8be9f54234aac3ba27e41f1b3dac856d36ef0  release-evidence/manifest.json
dcdbc5902c6a642e6fe97693ea0fa6aa892f7c4b12873a0dbc93cc4ca1f3b82d  release-verify.json
7557adc8ea544fb15193ed8cec18804a149a4df3e22597e2ad46801604a45d84  release-verification/release-attestation.json
1aaa814b647daec3df1ed1d9482f6e87ac3200e36c0f68f3e912c6f35faa9e1f  release-verification/release-attestation.json.sig
633dd0b3be0e11891eee3126d5db07ae5c4e1a4a02b1f49a354f5fb2a3c6e538  release-verification/policy.json
53d6e58465e29141d53dd719fe400597b5a496af4062ca5f3db996187e266b67  release-verification/witnesses/aspen1-external-witness.json
e321c71f8aeb50fffd1ffeafe209c091abd0e2406cd6c1f6155967fb7ec1f3b4  release-verification/witnesses/aspen1-external-witness.json.sig
655c95bbcd5328977c9f6762431e54c21f23f9539b04afdce738a75d9010adc5  request-digest-and-private-check.txt
107552606f348388658c7a7a66fafac1287069f82a2ccb07a19100cc3198693a  source-archive-policy-check.txt
1bc24b701a653193fa1f8fa0900b67d54125caf308e3e168bc8bee95a15c34bd  external-witness-aspen1-2026-07-02/status.txt
102a65bd9c9421d643c436f75bb08eae3cc0a53c0a238e7fedffb9a456e56d7e  external-witness-aspen1-2026-07-02/stdout.txt
25edcfc85e911b46880ed560065eaf39452ef2c63f3712998c5f3e4540fb3231  external-witness-aspen1-2026-07-02/stderr.txt
7c82614514556bf11e2c7726aa0cf59fdfe34c9816a0a18372517f308208ff5b  external-witness-aspen1-2026-07-02/witness-rebuild-audit-meta.json
767fd92f118c7c76e2ca3ed9569a8163e6988ccdec87e389c49fa6136d2908cd  witness-import.txt
8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260  final-release-verify.txt
8bec405f44936333e22858cf0551fc69812ac6264d7e42d7728c215af637a260  final-release-verify.json
d47bc3cdb4cc8251bf6e1881fee2f72142ee7893132aa45c2b2967747237d75e  negative-missing-signature/witness-import-missing-signature.transcript.txt
```
