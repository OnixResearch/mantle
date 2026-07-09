# mantle-cairn-release-evidence-handoff validation transcript
2026-07-09T19:12:44Z

```text
$ nix develop -c rustfmt --check --config skip_children=true src/main.rs src/cairn_release_handoff.rs
```

```text
$ nix develop -c cargo test -p mantle --bin mantle cairn_release_handoff

running 2 tests
test cairn_release_handoff::tests::accepts_complete_cairn_handoff_fixture ... ok
test cairn_release_handoff::tests::rejects_invalid_cairn_handoff_fixture_matrix ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1276 filtered out; finished in 0.00s

```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 22,
  "valid": true
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate proposal mantle-cairn-release-evidence-handoff --root .
{
  "change": "mantle-cairn-release-evidence-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "0501a3ea23664dfbe0abf179c3d9fa712965b841e63e041b8888fbaab442aaa7",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "fdc173d8ff1df65b1e81eb412854534cd9839d819ad2d9d4aaab0d9500b4b35c",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate design mantle-cairn-release-evidence-handoff --root .
{
  "change": "mantle-cairn-release-evidence-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e8dcfdccf1c28aed601f58c90ab94e34c39cbcd7ac475b8a17feee3cc0a84d53",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "483ebc59449a3bbbeb259e6b6f558464e10dea0abf7c66ab3405a5cbb7967219",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate tasks mantle-cairn-release-evidence-handoff --root .
{
  "change": "mantle-cairn-release-evidence-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d357257f9962ac95fb93668480acf030c4f05b442a2d60edfa0707d00c72c996",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "d85e13e7760403306b4064616ed9bc3f40d96d78424107a33f7c9aff2f12b890",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## accepted spec manual merge note
Cairn sync executed with reasons=[] but did not mutate the full-spec-shaped delta, so the Cairn handoff requirements were manually appended to cairn/specs/release-provenance/spec.md before archive.
## post-merge accepted requirement IDs
```text
186:r[mantle.release_provenance.cairn_evidence_handoff.contract] Mantle MUST validate Cairn release evidence handoff rows with explicit artifact id, role, schema, artifact digest, Cairn policy digest, release-readiness id, coverage ids, and non-claims.
189:r[mantle.release_provenance.cairn_evidence_handoff.fixtures.positive]
195:r[mantle.release_provenance.cairn_evidence_handoff.fixtures.negative]
201:r[mantle.release_provenance.cairn_evidence_handoff.validation] Mantle MUST integrate Cairn handoff validation as release external evidence checking over loaded rows while file reads, digest measurement, and Cairn export production remain outside the pure core.
204:r[mantle.release_provenance.cairn_evidence_handoff.docs]
211:r[mantle.release_provenance.cairn_evidence_handoff.final_validation] The Cairn handoff change MUST include positive and negative fixtures plus focused validation evidence before archive.
```
## post-merge cairn validate
```text
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 22,
  "valid": true
}
```
## post-merge cairn gate proposal
```text
{
  "change": "mantle-cairn-release-evidence-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "0501a3ea23664dfbe0abf179c3d9fa712965b841e63e041b8888fbaab442aaa7",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "fdc173d8ff1df65b1e81eb412854534cd9839d819ad2d9d4aaab0d9500b4b35c",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate design
```text
{
  "change": "mantle-cairn-release-evidence-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e8dcfdccf1c28aed601f58c90ab94e34c39cbcd7ac475b8a17feee3cc0a84d53",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "483ebc59449a3bbbeb259e6b6f558464e10dea0abf7c66ab3405a5cbb7967219",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate tasks
```text
{
  "change": "mantle-cairn-release-evidence-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d357257f9962ac95fb93668480acf030c4f05b442a2d60edfa0707d00c72c996",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "d85e13e7760403306b4064616ed9bc3f40d96d78424107a33f7c9aff2f12b890",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
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
## post-archive status
```text
## main...origin/main [ahead 5]
 M README.md
 D cairn/changes/mantle-cairn-release-evidence-handoff/design.md
 D cairn/changes/mantle-cairn-release-evidence-handoff/proposal.md
 D cairn/changes/mantle-cairn-release-evidence-handoff/specs/release-provenance/spec.md
 D cairn/changes/mantle-cairn-release-evidence-handoff/tasks.md
 M cairn/specs/release-provenance/spec.md
 M src/main.rs
?? cairn/archive/2026-07-09-mantle-cairn-release-evidence-handoff/
?? cairn/changes/adopt-cap-std-release-boundaries/
?? cairn/changes/nix-evidence-core/
?? docs/cairn-release-evidence-handoff.md
?? src/cairn_release_handoff.rs
```
