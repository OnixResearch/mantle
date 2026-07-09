# adopt-cap-std-release-boundaries validation transcript
2026-07-09T19:23:46Z

```text
$ nix develop -c rustfmt --check --config skip_children=true src/main.rs src/release_capability.rs
```

```text
$ nix develop -c cargo test -p mantle --bin mantle release_capability

running 4 tests
test release_capability::tests::accepts_relative_paths_under_declared_roots ... ok
test release_capability::tests::rejects_path_and_authority_negative_fixtures ... ok
test release_capability::tests::cap_std_root_reads_and_writes_relative_files ... ok
test release_capability::tests::cap_std_root_rejects_symlink_escape_reads ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1280 filtered out; finished in 0.01s

```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- validate --root .
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

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate proposal adopt-cap-std-release-boundaries --root .
{
  "change": "adopt-cap-std-release-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "386da2e21a8e5d8b197b1f85b91646690f522f257fb8e1479d498b8f88612ebe",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "d9531b80dfa038c0377db5c0aefa8fed16cded055415d11b6395a3ed778ade9b",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate design adopt-cap-std-release-boundaries --root .
{
  "change": "adopt-cap-std-release-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b085b3ab100b808d7c20b17221b9b4fa737e72886ddc8933bf448b61a4488f77",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "514c38b800908c2a45048de242dcfb47d5cff111731d191db58374f2a5665aca",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate tasks adopt-cap-std-release-boundaries --root .
{
  "change": "adopt-cap-std-release-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6c697760c6357cf4c1d84c8763873cadae59c51932aaf58b61bbce9a4ac5f0f0",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "1c6f7a160560c2c104413721ec667e180dd5192e4d796abb00018b867fe7cc3d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## accepted spec manual merge note
Cairn sync executed with reasons=[] but did not mutate the full-spec-shaped delta, so the cap-std release boundary requirements were manually appended to cairn/specs/release-provenance/spec.md before archive.
## post-merge accepted requirement IDs
```text
252:r[mantle.release_provenance.cap_std_boundary.dependency] Mantle MUST use `cap-std` only in crates or modules that own filesystem shell/adaptor behavior and MUST keep pure release planning cores free of ambient filesystem authority.
260:r[mantle.release_provenance.cap_std_boundary.root_wrappers] Mantle MUST expose typed capability roots for release evidence, witness rebuild, bootstrap, build artifact, and store roots.
263:r[mantle.release_provenance.cap_std_boundary.tests.positive]
269:r[mantle.release_provenance.cap_std_boundary.conversion] Mantle MUST convert targeted release path opens to capability-relative operations without moving filesystem authority into pure planning logic.
272:r[mantle.release_provenance.cap_std_boundary.tests.negative]
278:r[mantle.release_provenance.cap_std_boundary.docs] Mantle docs MUST describe the local filesystem-authority boundary and preserve release-evidence non-claims.
281:r[mantle.release_provenance.cap_std_boundary.validation]
```
## post-merge cairn validate
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
## post-merge cairn gate proposal
```text
{
  "change": "adopt-cap-std-release-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "386da2e21a8e5d8b197b1f85b91646690f522f257fb8e1479d498b8f88612ebe",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "d9531b80dfa038c0377db5c0aefa8fed16cded055415d11b6395a3ed778ade9b",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate design
```text
{
  "change": "adopt-cap-std-release-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b085b3ab100b808d7c20b17221b9b4fa737e72886ddc8933bf448b61a4488f77",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "514c38b800908c2a45048de242dcfb47d5cff111731d191db58374f2a5665aca",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate tasks
```text
{
  "change": "adopt-cap-std-release-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6c697760c6357cf4c1d84c8763873cadae59c51932aaf58b61bbce9a4ac5f0f0",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "1c6f7a160560c2c104413721ec667e180dd5192e4d796abb00018b867fe7cc3d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
```
## post-archive status
```text
## main...origin/main [ahead 7]
 M Cargo.lock
 M Cargo.toml
 M README.md
 M cairn/specs/release-provenance/spec.md
 M src/main.rs
?? cairn/archive/2026-07-09-adopt-cap-std-release-boundaries/
?? docs/release-capability-boundaries.md
?? src/release_capability.rs
```
