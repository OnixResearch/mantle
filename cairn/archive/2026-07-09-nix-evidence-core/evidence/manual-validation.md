# nix-evidence-core validation transcript
2026-07-09T19:17:56Z

```text
$ nix develop -c rustfmt --check --config skip_children=true src/main.rs src/nix_evidence_core.rs
```

```text
$ nix develop -c cargo test -p mantle --bin mantle nix_evidence_core

running 2 tests
test nix_evidence_core::tests::accepts_supported_nix_evidence_adapter_rows ... ok
test nix_evidence_core::tests::rejects_invalid_nix_evidence_fixture_matrix ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1278 filtered out; finished in 0.00s

```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 21,
  "valid": true
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate proposal nix-evidence-core --root .
{
  "change": "nix-evidence-core",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "68c925cb17752cea0ae6f6e76fb0749fd3fb3f203dfebec0eb34f351fd6c3511",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "97eccc8867757c355159e91bbc7bd36f08ce32ddb7e243e526c186911ccc1b32",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate design nix-evidence-core --root .
{
  "change": "nix-evidence-core",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "5a66a14230bf3d2c8d8e8d4da88c2f29215f6135b6ff14d9b8cebdf295339f08",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "3a85e0af3a588a5a609e4f877f10c5a297d7cadf65a5c89e7ddfdb4800e352d4",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate tasks nix-evidence-core --root .
{
  "change": "nix-evidence-core",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "32e423ded9a2e4f7ece5965800f2768cfc7e639c983e4441e1e485984811b33f",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "17ab85df57add8f72cfbf9a6bd074ea2fa95c8a9a1cb7cc02e3f95d5fb78fa65",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## accepted spec manual merge note
Cairn sync executed with reasons=[] but did not mutate the full-spec-shaped delta, so the Nix evidence requirements were manually appended to cairn/specs/release-provenance/spec.md before archive.
## post-merge accepted requirement IDs
```text
219:r[mantle.release_provenance.nix_evidence_core.contract] Mantle MUST normalize Nix-related release evidence rows with store path ref, derivation identity, output name, realization role, caveat, artifact digest, and non-claim fields.
222:r[mantle.release_provenance.nix_evidence_core.fixtures.positive]
228:r[mantle.release_provenance.nix_evidence_core.fixtures.negative]
234:r[mantle.release_provenance.nix_evidence_core.validation] Mantle MUST validate normalized Nix evidence in a pure core and keep build/evaluation shell work, file reads, and source adapter parsing outside that core.
237:r[mantle.release_provenance.nix_evidence_core.adapters]
243:r[mantle.release_provenance.nix_evidence_core.docs] Mantle operator docs MUST describe Nix evidence as realization identity only and name downstream migration boundaries.
246:r[mantle.release_provenance.nix_evidence_core.final_validation]
```
## post-merge cairn validate
```text
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 21,
  "valid": true
}
```
## post-merge cairn gate proposal
```text
{
  "change": "nix-evidence-core",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "68c925cb17752cea0ae6f6e76fb0749fd3fb3f203dfebec0eb34f351fd6c3511",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "97eccc8867757c355159e91bbc7bd36f08ce32ddb7e243e526c186911ccc1b32",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate design
```text
{
  "change": "nix-evidence-core",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "5a66a14230bf3d2c8d8e8d4da88c2f29215f6135b6ff14d9b8cebdf295339f08",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "3a85e0af3a588a5a609e4f877f10c5a297d7cadf65a5c89e7ddfdb4800e352d4",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate tasks
```text
{
  "change": "nix-evidence-core",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "32e423ded9a2e4f7ece5965800f2768cfc7e639c983e4441e1e485984811b33f",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "17ab85df57add8f72cfbf9a6bd074ea2fa95c8a9a1cb7cc02e3f95d5fb78fa65",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 20,
  "valid": true
}
```
## post-archive status
```text
## main...origin/main [ahead 6]
 M README.md
 M cairn/specs/release-provenance/spec.md
 M src/main.rs
?? cairn/archive/2026-07-09-nix-evidence-core/
?? cairn/changes/
?? docs/nix-evidence-core.md
?? src/nix_evidence_core.rs
```
