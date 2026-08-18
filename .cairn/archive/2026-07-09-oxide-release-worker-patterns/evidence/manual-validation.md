# oxide-release-worker-patterns validation transcript
2026-07-09T18:45:34Z

```text
$ nix develop -c rustfmt --check --config skip_children=true crates/crunch-release-core/src/lib.rs crates/crunch-release-core/src/oxide_worker.rs
```

```text
$ nix develop -c cargo test -p crunch-release-core oxide_release_worker --lib

running 2 tests
test oxide_worker::tests::accepts_valid_oxide_release_worker_fixture ... ok
test oxide_worker::tests::rejects_invalid_oxide_release_worker_fixture_matrix ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 134 filtered out; finished in 0.00s

```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- validate --root .
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

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate proposal oxide-release-worker-patterns --root .
{
  "change": "oxide-release-worker-patterns",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8974da0f4064e1026bcd6ba409f39482e4030ec0949182aef242513e047c49fb",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "8adf98e0892109891afc9c4a5f6864e1039442f46cc85f23f0d26c080f707562",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate design oxide-release-worker-patterns --root .
{
  "change": "oxide-release-worker-patterns",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "af68e55e69c1ff95abc5dc1cf8609dd77dd895ee90089b6e604a7d1f9b7d66c0",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "3cb8c859d33643019ba6b71e7a5eaf28f7e5190218b5b8702bd65eae9c4a460c",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate tasks oxide-release-worker-patterns --root .
{
  "change": "oxide-release-worker-patterns",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "25efeec23143d50eea1123d0765a75b1be5f76b37e5876fbea59b6f402599c46",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "365682889990ee842fff1a190abcdf81048d3d9a407faa4ca6e202fc09cfc55e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## accepted spec manual merge note
Cairn sync executed with reasons=[] but did not mutate the full-spec-shaped delta, so the Oxide release/worker requirements were manually appended to cairn/specs/verification-evidence/spec.md before archive.
## post-merge accepted requirement IDs
```text
1189:r[mantle.verification_evidence.oxide_release_worker.reference_inventory] Mantle MUST record reference-only intake notes for each Oxide release, worker, or cancellation-safety repository used to shape Mantle evidence contracts.
1192:r[mantle.verification_evidence.oxide_release_worker.reference_inventory.boundary]
1198:r[mantle.verification_evidence.oxide_release_worker.reference_inventory.non_authority]
1204:r[mantle.verification_evidence.oxide_release_worker.tuf_bundles] Mantle SHOULD define TUF-style release repository profiles for signed metadata, target manifests, artifact tags, compatibility checks, and trust-root handling.
1207:r[mantle.verification_evidence.oxide_release_worker.tuf_bundles.signed_targets]
1213:r[mantle.verification_evidence.oxide_release_worker.tuf_bundles.tag_mismatch]
1219:r[mantle.verification_evidence.oxide_release_worker.ephemeral_workers] Mantle MUST treat ephemeral build workers as bounded evidence producers with explicit job, artifact, log, and cleanup receipts.
1222:r[mantle.verification_evidence.oxide_release_worker.ephemeral_workers.complete_receipt]
1228:r[mantle.verification_evidence.oxide_release_worker.ephemeral_workers.policy_override]
1234:r[mantle.verification_evidence.oxide_release_worker.cancel_safety] Mantle MUST model async cancellation as an explicit worker outcome that preserves cleanup and evidence integrity or fails closed.
1237:r[mantle.verification_evidence.oxide_release_worker.cancel_safety.cleanup]
1243:r[mantle.verification_evidence.oxide_release_worker.cancel_safety.mid_upload]
1249:r[mantle.verification_evidence.oxide_release_worker.validation] Mantle MUST validate release repository, ephemeral worker, and cancellation-safety profiles with positive and negative fixtures before implementation adoption.
1252:r[mantle.verification_evidence.oxide_release_worker.validation.positive]
1258:r[mantle.verification_evidence.oxide_release_worker.validation.negative]
```
## post-merge cairn validate
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
## post-merge cairn gate proposal
```text
{
  "change": "oxide-release-worker-patterns",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8974da0f4064e1026bcd6ba409f39482e4030ec0949182aef242513e047c49fb",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "8adf98e0892109891afc9c4a5f6864e1039442f46cc85f23f0d26c080f707562",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate design
```text
{
  "change": "oxide-release-worker-patterns",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "af68e55e69c1ff95abc5dc1cf8609dd77dd895ee90089b6e604a7d1f9b7d66c0",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "3cb8c859d33643019ba6b71e7a5eaf28f7e5190218b5b8702bd65eae9c4a460c",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate tasks
```text
{
  "change": "oxide-release-worker-patterns",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "25efeec23143d50eea1123d0765a75b1be5f76b37e5876fbea59b6f402599c46",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "365682889990ee842fff1a190abcdf81048d3d9a407faa4ca6e202fc09cfc55e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
```text
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 25,
  "valid": true
}
```
## post-archive status
```text
## main...origin/main [ahead 1]
 M README.md
 M cairn/specs/verification-evidence/spec.md
 M crates/crunch-release-core/src/lib.rs
?? cairn/archive/2026-07-09-oxide-release-worker-patterns/
?? cairn/changes/adopt-cap-std-release-boundaries/
?? cairn/changes/nix-evidence-core/
?? cairn/changes/rustc-dev-guide-planning-boundaries/
?? crates/crunch-release-core/src/oxide_worker.rs
?? docs/oxide-release-worker-patterns.md
```
