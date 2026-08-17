# Release reproducibility transcript: provider-fixed-point-release-evidence-2026-06-25

Date: 2026-06-25
Change: `record-deterministic-release-evidence`
Requirement: r[verification_evidence.release_reproducibility_transcripts]

## Scope and bounded claim

This transcript durably records current evidence for release bundle
`target/release-evidence/provider-fixed-point-release-evidence-2026-06-25`.
The bounded claim is: the packaged `binaries/01-stage2-mantle` artifact rebuilt
twice from the recorded release inputs under recorded `mantle-proof-sandbox-v1`
profiles, the BLAKE3 digest sets matched, and the bundled provider fixed-point
proof validated.

Non-claims:

- No full bootstrap reproducibility claim.
- No compiler correctness claim.
- No full Cargo compatibility claim.
- No deployability or physical-target determinism claim.
- No claim that generated `target/` proof payloads are tracked repository state.

## Generated paths intentionally left untracked

- Release bundle: `target/release-evidence/provider-fixed-point-release-evidence-2026-06-25`
- Fresh reproduce output: `target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs`
- Fresh deterministic proof sidecars: `target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof`
- Fresh verifier receipt: `target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-verify.json`

Only this concise lifecycle evidence text is intended to be tracked.

## Key digests and statuses

- Release id: `provider-fixed-point-release-evidence-2026-06-25`
- Reproduce report digest: `81124b3878602ec5fdcf98f20efd2382187942c21df548926b4de45f5d9d54bb`
- Deterministic proof digest: `c2a131abe400a9e56e5e6507c6cc6a304a420f98ce7d4d9981f174f0c21c8773`
- Sandbox isolation evidence digest: `7f1eb23561dcf7dc68ca7b9d2c268326cdfef068a1cbcf5034d4ff2ae655ba31`
- Rebuilt artifact digest set: `binaries/01-stage2-mantle = 84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd`
- Provider fixed-point proof status: `valid`
- Provider fixed-point proof source: `bundled`
- Provider fixed-point proof digest: `fe2cf79289372b1231ab95cf0d7161218ce6740f80e864b137858d20c679df59`
- Deterministic verifier status: `eligible`
- Reproducibility status: `matched`

## Evidence commands and output

### 1. Reproduce with deterministic proof runs

Command (pueue task 179):

```sh
nix shell nixpkgs#bubblewrap -c ./target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/binaries/01-stage2-mantle --json release reproduce target/release-evidence/provider-fixed-point-release-evidence-2026-06-25 --rebuild-output-dir target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs --rebuild-command /nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox --rebuild-arg sh --rebuild-arg /home/brittonr/git/mantle/target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/rebuild-stage2-mantle-copy.sh --report-path target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs/reproducibility-report.json --deterministic-proof-runs 2 --deterministic-proof-dir target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof
```

Output:

```json
{"deterministic_proof_blockers":[],"deterministic_proof_digest_blake3":"c2a131abe400a9e56e5e6507c6cc6a304a420f98ce7d4d9981f174f0c21c8773","deterministic_proof_path":"/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-build-proof.json","deterministic_proof_run_roots":[{"output_root_identity":"/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/run-000/outputs","output_store_paths":["/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/run-000/store"],"run_id":"run-000","sandbox_profile_identity":"mantle-proof-sandbox-v1:b4a11ce65dfed1219dc6daefcc637d81ce40885ace8f034fa753e657904c09f9"},{"output_root_identity":"/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/run-001/outputs","output_store_paths":["/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/run-001/store"],"run_id":"run-001","sandbox_profile_identity":"mantle-proof-sandbox-v1:c4028fb681200feed0e58d6d667a85561733305d5071697133b0c474ad559f35"}],"deterministic_proof_sandbox_profiles":["mantle-proof-sandbox-v1:b4a11ce65dfed1219dc6daefcc637d81ce40885ace8f034fa753e657904c09f9","mantle-proof-sandbox-v1:c4028fb681200feed0e58d6d667a85561733305d5071697133b0c474ad559f35"],"deterministic_proof_unit":{"output_identities":["binaries/01-stage2-mantle"],"selected_provider_kind":"legacy-fetch","source_blake3":"e831802092c14e9d5cc3c53d5f40accc3ba0edff977b63e5d05678bbdae72481","target_artifact_identity":"release:provider-fixed-point-release-evidence-2026-06-25","toolchain_stage_roots":["staged-source=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmprM89kH/store/brnjbzjaazy2j8kk5czadfs6yv1gnlhn-mantle-src","stage2-binary=84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd","prerequisite-inventory=3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb"],"vendor_blake3":"45901c49cd828f8bcd4d6d1681b630e5e97d1f2d4a0ac06bf36dd7d6c419fcfb"},"deterministic_proof_verdict":"self-rebuild-match","deterministic_sandbox_isolation_evidence_digest_blake3":"7f1eb23561dcf7dc68ca7b9d2c268326cdfef068a1cbcf5034d4ff2ae655ba31","deterministic_sandbox_isolation_evidence_path":"/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-sandbox-isolation-evidence.json","matched_count":1,"mismatched_count":0,"missing_count":0,"release_id":"provider-fixed-point-release-evidence-2026-06-25","report_digest_blake3":"81124b3878602ec5fdcf98f20efd2382187942c21df548926b4de45f5d9d54bb","report_path":"/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs/reproducibility-report.json"}
```

### 2. Required verifier with deterministic and provider fixed-point gates

Command (pueue task 117):

```sh
./target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/binaries/01-stage2-mantle --json release verify target/release-evidence/provider-fixed-point-release-evidence-2026-06-25 --require-deterministic-release --deterministic-proof target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-build-proof.json --deterministic-sandbox-isolation-evidence target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-sandbox-isolation-evidence.json --require-provider-fixed-point-proof | tee target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-verify.json
```

Output:

```json
{"deterministic_release":{"blockers":[],"eligible":true,"proof_digest_blake3":"c2a131abe400a9e56e5e6507c6cc6a304a420f98ce7d4d9981f174f0c21c8773","proof_path":"/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-build-proof.json","sandbox_isolation_evidence_digest_blake3":"7f1eb23561dcf7dc68ca7b9d2c268326cdfef068a1cbcf5034d4ff2ae655ba31","sandbox_isolation_evidence_path":"/home/brittonr/git/mantle/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-sandbox-isolation-evidence.json","status":"eligible"},"kind":"mantle-release-verify-v1","manifest":{"binaries":[{"digest_blake3":"84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd","kind":"file","relative_path":"binaries/01-stage2-mantle","size_bytes":63289920}],"claim_scope":"packaged-integrity-evidence","prerequisite_inventory":{"digest_blake3":"3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb","kind":"file","relative_path":"proof/inventory.md","size_bytes":14244},"proof_bundle":{"digest_blake3":"45901c49cd828f8bcd4d6d1681b630e5e97d1f2d4a0ac06bf36dd7d6c419fcfb","kind":"directory","relative_path":"proof/self-hosting","size_bytes":127242957},"proof_linkage":{"prerequisite_inventory_digest_blake3":"3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb","proof_bundle_schema":"mantle-self-hosting-proof-v2","proof_manifest_digest_blake3":"721443ada020a5845d12883f0655e4b42cd574362620e17e262d0f1949a0c499","proof_mode":"fixed-point","release_id":"provider-fixed-point-release-evidence-2026-06-25","selected_provider_kind":"legacy-fetch","source_archive_digest_blake3":"e831802092c14e9d5cc3c53d5f40accc3ba0edff977b63e5d05678bbdae72481","stage2_binary_digest_blake3":"84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd","staged_source":"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmprM89kH/store/brnjbzjaazy2j8kk5czadfs6yv1gnlhn-mantle-src"},"provider_fixed_point_proof":{"digest_blake3":"fe2cf79289372b1231ab95cf0d7161218ce6740f80e864b137858d20c679df59","evidence_role":"cargo-free-source-built-handoff-evidence","kind":"directory","relative_path":"proof/provider-fixed-point","size_bytes":2790890830},"release_id":"provider-fixed-point-release-evidence-2026-06-25","schema":"mantle-release-evidence-v1","source_archive":{"digest_blake3":"e831802092c14e9d5cc3c53d5f40accc3ba0edff977b63e5d05678bbdae72481","kind":"file","relative_path":"source/.tmp4us1I2","size_bytes":935829504},"workflow":{"command":"./scripts/prove-self-hosting.sh","version":"mantle-self-hosting-proof-v2"}},"provider_fixed_point_proof":{"blockers":[],"bounded_evidence_role":"cargo-free-source-built-handoff-evidence","closure_policy_digest_blake3":"c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093","meta_digest_blake3":"6c575595b3bfe58fb464c99f21621b6ff2fff208283f1d314e9bda61b2c0025d","non_claims":["not-crunch-bootstrap","not-release-reproducibility","not-full-cargo-compatibility"],"proof_artifact_digest_blake3":"fe2cf79289372b1231ab95cf0d7161218ce6740f80e864b137858d20c679df59","proof_dir":"/home/brittonr/git/mantle/target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/proof/provider-fixed-point","proof_source":"bundled","stage1_unit_count":686,"stage2_unit_count":686,"stage_binary_digest_blake3":"288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3","status":"valid","valid":true},"release_id":"provider-fixed-point-release-evidence-2026-06-25","reproducibility_report":{"digest_blake3":"54aa592211670ee2ffae57122112925248fafa8333862ba1eeb93d97765b0bb5","path":"/home/brittonr/git/mantle/target/release-evidence/provider-fixed-point-release-evidence-2026-06-25/reproducibility/reproducibility-report.json"},"reproducibility_status":"matched"}
```

### 3. Receipt checker

Command (pueue task 118):

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig cargo -Zscript scripts/check-real-release-determinism-receipt.rs --proof-dir target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof --verify-receipt target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-verify.json target/release-evidence/provider-fixed-point-release-evidence-2026-06-25
```

Output:

```text
real release determinism proof receipt valid
  release: provider-fixed-point-release-evidence-2026-06-25
  provider: legacy-fetch
  source BLAKE3: e831802092c14e9d5cc3c53d5f40accc3ba0edff977b63e5d05678bbdae72481
  proof bundle BLAKE3: 45901c49cd828f8bcd4d6d1681b630e5e97d1f2d4a0ac06bf36dd7d6c419fcfb
  artifact digests: [("binaries/01-stage2-mantle", "84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd")]
  proof: target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-build-proof.json
  sandbox evidence: target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-sandbox-isolation-evidence.json
  verify: target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-verify.json
  bounded claim: packaged release artifacts rebuilt twice from recorded inputs under recorded mantle-proof-sandbox-v1: profiles with matching BLAKE3 digest sets; this is not a full-bootstrap reproducibility claim
```

## Final validation

### Cairn validation

Command (pueue task 116):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root $PWD
```

Output:

```json
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 9,
  "valid": true
}
```

### Task gate

Command (pueue task 117):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks record-deterministic-release-evidence --root $PWD
```

Output excerpt:

```json
{
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

### Tracked status

Command (pueue task 118):

```sh
git status --short --branch
```

Output:

```text
## main...origin/main
 M cairn/changes/record-deterministic-release-evidence/tasks.md
?? cairn/changes/record-deterministic-release-evidence/evidence/
```

## Post-archive validation

### Cairn validation after archive execution

Command (pueue task 129):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root $PWD
```

Output:

```json
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 8,
  "valid": true
}
```

### Tracked status after archive execution

Command (pueue task 130):

```sh
git status --short --branch
```

Output:

```text
## main...origin/main
 D cairn/changes/record-deterministic-release-evidence/design.md
 D cairn/changes/record-deterministic-release-evidence/proposal.md
 D cairn/changes/record-deterministic-release-evidence/specs/verification-evidence/spec.md
 D cairn/changes/record-deterministic-release-evidence/tasks.md
 M cairn/specs/verification-evidence/spec.md
?? cairn/archive/2026-06-25-record-deterministic-release-evidence/
```
