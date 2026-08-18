# preserves-release-evidence-carriers validation transcript
2026-07-09T19:07:51Z

```text
$ nix develop -c rustfmt --check --config skip_children=true src/main.rs src/preserves_release_carrier.rs
```

```text
$ nix develop -c cargo test -p mantle --bin mantle preserves_release_carrier

running 2 tests
test preserves_release_carrier::tests::accepts_opaque_and_adapter_backed_preserves_carriers ... ok
test preserves_release_carrier::tests::rejects_invalid_preserves_carrier_fixture_matrix ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1274 filtered out; finished in 0.00s

```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 23,
  "valid": true
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate proposal preserves-release-evidence-carriers --root .
{
  "change": "preserves-release-evidence-carriers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ffa4bb8b8800aefb4600129660e0069224f07111184f125a78f34fa06585d645",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "356103b8de54d9baeb23647a2703d69a2704f772ef60494dfdc3ad8179df9e38",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate design preserves-release-evidence-carriers --root .
{
  "change": "preserves-release-evidence-carriers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "39ae728072145260429a552498bd7ccd9bdf9dee89f2061bdac1b6ab90409fdf",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "0093f7474f4ff56d50bff999a62b3b784ae592fae80ec8a3920261c56c83f61d",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate tasks preserves-release-evidence-carriers --root .
{
  "change": "preserves-release-evidence-carriers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "762538be7f9bc8205f6cb11dc57efdf7133596206efd0eae76b5dbc483a347ec",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "bdb2235b78a5cedde6555b5e2f5e76783c58d5c7869f0052f6b6a3c42360ee73",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## accepted spec manual merge note
Cairn sync executed with reasons=[] but did not mutate the full-spec-shaped delta, so the Preserves carrier requirements were manually appended to cairn/specs/release-provenance/spec.md before archive.
## post-merge accepted requirement IDs
```text
153:r[mantle.release_provenance.preserves_carriers.contract] Mantle MUST validate Preserves release evidence carrier rows with explicit role, schema, payload digest, canonical digest, adapter identity when present, and bounded non-claims.
156:r[mantle.release_provenance.preserves_carriers.fixtures.positive]
162:r[mantle.release_provenance.preserves_carriers.fixtures.negative]
168:r[mantle.release_provenance.preserves_carriers.validation] Mantle MUST integrate Preserves carrier validation as release external evidence checking over loaded rows while keeping file I/O, digesting, and payload acquisition in the shell.
171:r[mantle.release_provenance.preserves_carriers.docs]
178:r[mantle.release_provenance.preserves_carriers.final_validation] The Preserves carrier change MUST include positive and negative fixtures plus focused validation evidence before archive.
```
## post-merge cairn validate
```text
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 23,
  "valid": true
}
```
## post-merge cairn gate proposal
```text
{
  "change": "preserves-release-evidence-carriers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ffa4bb8b8800aefb4600129660e0069224f07111184f125a78f34fa06585d645",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "356103b8de54d9baeb23647a2703d69a2704f772ef60494dfdc3ad8179df9e38",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate design
```text
{
  "change": "preserves-release-evidence-carriers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "39ae728072145260429a552498bd7ccd9bdf9dee89f2061bdac1b6ab90409fdf",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "0093f7474f4ff56d50bff999a62b3b784ae592fae80ec8a3920261c56c83f61d",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate tasks
```text
{
  "change": "preserves-release-evidence-carriers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "762538be7f9bc8205f6cb11dc57efdf7133596206efd0eae76b5dbc483a347ec",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "bdb2235b78a5cedde6555b5e2f5e76783c58d5c7869f0052f6b6a3c42360ee73",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
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
## post-archive status
```text
## main...origin/main [ahead 4]
 M README.md
 D cairn/changes/preserves-release-evidence-carriers/design.md
 D cairn/changes/preserves-release-evidence-carriers/proposal.md
 D cairn/changes/preserves-release-evidence-carriers/specs/release-provenance/spec.md
 D cairn/changes/preserves-release-evidence-carriers/tasks.md
 M cairn/specs/release-provenance/spec.md
 M src/main.rs
?? cairn/archive/2026-07-09-preserves-release-evidence-carriers/
?? cairn/changes/adopt-cap-std-release-boundaries/
?? cairn/changes/nix-evidence-core/
?? docs/preserves-release-carriers.md
?? src/preserves_release_carrier.rs
```
