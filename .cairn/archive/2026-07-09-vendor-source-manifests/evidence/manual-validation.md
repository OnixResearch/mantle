# vendor-source-manifests validation transcript
2026-07-09T18:52:45Z

```text
$ nix develop -c rustfmt --check --config skip_children=true src/main.rs src/vendor_source_manifest.rs
```

```text
$ nix develop -c cargo test -p mantle --bin mantle vendor_source_manifest

running 3 tests
test vendor_source_manifest::tests::rejects_invalid_vendor_manifest_fixture_matrix ... ok
test vendor_source_manifest::tests::measuring_rejects_path_escape ... ok
test vendor_source_manifest::tests::validates_fresh_vendor_manifest_and_measured_file ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1269 filtered out; finished in 0.00s

```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- validate --root .
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

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate proposal vendor-source-manifests --root .
{
  "change": "vendor-source-manifests",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "2ac8e7cddcb8182e6c95a7f3e41df5ebffa1947af1b8b4dcf528e804da54aad6",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "870b4f578afbfbfafac50cff3679766ff7a6df1e471dfc3e725a9995e2cf2bdc",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate design vendor-source-manifests --root .
{
  "change": "vendor-source-manifests",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "dea526d210eb98bf9d3776d600eadf454ce89e16a47907fcf8354d338c9303dd",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "135c81ec0ba8a01719b13f77fea904d186aed3eb6dd25957effafc307d81dc8c",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate tasks vendor-source-manifests --root .
{
  "change": "vendor-source-manifests",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ec492dafb8ad4b8b4b0e759d73c0782cea8003bccbd4aa565cd7aa2eb3f7b94f",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "eec4ddd72de91f3cf583a721f03f7c805d7a09e15868190f667034552c491438",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## accepted spec manual merge note
Cairn sync executed with reasons=[] but did not mutate the full-spec-shaped delta, so the vendor-source requirements were manually appended to cairn/specs/source-transports/spec.md before archive.
## post-merge accepted requirement IDs
```text
196:r[mantle.source_transports.vendor_source_manifests.contract] Mantle MUST support a vendor source manifest contract that records upstream repo, revision, filter, selected paths, measured identities, local edits, refresh command, and non-claims.
199:r[mantle.source_transports.vendor_source_manifests.fixtures.positive]
205:r[mantle.source_transports.vendor_source_manifests.fixtures.negative]
211:r[mantle.source_transports.vendor_source_manifests.validation] Mantle MUST validate vendor manifests over parsed rows and measured file identities.
214:r[mantle.source_transports.vendor_source_manifests.docs]
220:r[mantle.source_transports.vendor_source_manifests.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.
223:r[mantle.source_transports.vendor_source_manifests.final_validation.fixtures]
```
## post-merge cairn validate
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
## post-merge cairn gate proposal
```text
{
  "change": "vendor-source-manifests",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "2ac8e7cddcb8182e6c95a7f3e41df5ebffa1947af1b8b4dcf528e804da54aad6",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "870b4f578afbfbfafac50cff3679766ff7a6df1e471dfc3e725a9995e2cf2bdc",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate design
```text
{
  "change": "vendor-source-manifests",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "dea526d210eb98bf9d3776d600eadf454ce89e16a47907fcf8354d338c9303dd",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "135c81ec0ba8a01719b13f77fea904d186aed3eb6dd25957effafc307d81dc8c",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate tasks
```text
{
  "change": "vendor-source-manifests",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ec492dafb8ad4b8b4b0e759d73c0782cea8003bccbd4aa565cd7aa2eb3f7b94f",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "eec4ddd72de91f3cf583a721f03f7c805d7a09e15868190f667034552c491438",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
```text
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 24,
  "valid": true
}
```
## post-archive status
```text
## main...origin/main [ahead 2]
 M README.md
 D cairn/changes/vendor-source-manifests/design.md
 D cairn/changes/vendor-source-manifests/proposal.md
 D cairn/changes/vendor-source-manifests/specs/source-transports/spec.md
 D cairn/changes/vendor-source-manifests/tasks.md
 M cairn/specs/source-transports/spec.md
 M src/main.rs
?? cairn/archive/2026-07-09-vendor-source-manifests/
?? cairn/changes/adopt-cap-std-release-boundaries/
?? cairn/changes/nix-evidence-core/
?? cairn/changes/rustc-dev-guide-planning-boundaries/
?? docs/vendor-source-manifests.md
?? src/vendor_source_manifest.rs
```
