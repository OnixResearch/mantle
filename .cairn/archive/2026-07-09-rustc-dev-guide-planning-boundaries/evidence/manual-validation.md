# rustc-dev-guide-planning-boundaries validation transcript
2026-07-09T18:59:07Z

```text
$ nix develop -c rustfmt --check --config skip_children=true src/main.rs src/rustc_dev_guide.rs
```

```text
$ nix develop -c cargo test -p mantle --bin mantle rustc_dev_guide

running 2 tests
test rustc_dev_guide::tests::accepts_pinned_rustc_dev_guide_fixture ... ok
test rustc_dev_guide::tests::rejects_rustc_dev_guide_boundary_fixture_matrix ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1272 filtered out; finished in 0.00s

```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- validate --root .
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

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate proposal rustc-dev-guide-planning-boundaries --root .
{
  "change": "rustc-dev-guide-planning-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e1fb6f54557364c482713688408246aa306b85ee3b9608cd7527e13b6b2d995e",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "861040f1fbd29388f25f94f24810f44a3fb4827e60cbbbda48044ce15d0897fb",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate design rustc-dev-guide-planning-boundaries --root .
{
  "change": "rustc-dev-guide-planning-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8d7be198528de12374d520314498588e7a2282aba9848066955f5590e7935f64",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "7bbf7ec031c1f3e8919326bb5ab112777d1832e7ca53991e298587eff2d0612a",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

```text
$ nix run path:${ONIX_RESEARCH_ROOT:-$HOME/git/OnixResearch}/cairn#cairn -- gate tasks rustc-dev-guide-planning-boundaries --root .
{
  "change": "rustc-dev-guide-planning-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "2fc855c8365efdd16366e897345ea5189e778c63abce3493e0fe45ae28499b28",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "7a13101d272c6e8114494d3dad3d151c02217add6a9dba370e68133704dfc00a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## accepted spec manual merge note
Cairn sync executed with reasons=[] but did not mutate the full-spec-shaped delta, so the rustc-dev-guide requirements were manually appended to cairn/specs/rust-package-planning/spec.md before archive.
## post-merge accepted requirement IDs
```text
3965:r[rust_package_planning.rustc_dev_guide.reference_map] Mantle MUST use a pinned rustc-dev-guide reference map before making claim-bearing Rust planning, compiler-policy adapter, or source-built Rust provider statements from rustc internals documentation.
3968:r[rust_package_planning.rustc_dev_guide.reference_map.pinned]
3975:r[rust_package_planning.rustc_dev_guide.backend_invocation] Mantle MUST bind guide-backed backend invocation assumptions to explicit Rust planning evidence.
3984:r[rust_package_planning.rustc_dev_guide.compiler_policy_adapter] Mantle MUST keep compiler-policy adapter claims scoped to guide-backed HIR, MIR, driver, or invocation boundaries and separate from compiler or program correctness.
3993:r[rust_package_planning.rustc_dev_guide.source_provider_patch_plan] Mantle MUST bind source-built Rust provider patch-plan operations that rely on rustc internals to guide-backed source anchors and digest evidence.
4002:r[rust_package_planning.rustc_dev_guide.final_validation] The rustc-dev-guide planning-boundary change MUST include positive and negative fixtures plus focused validation evidence before archive.
```
## post-merge cairn validate
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
## post-merge cairn gate proposal
```text
{
  "change": "rustc-dev-guide-planning-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e1fb6f54557364c482713688408246aa306b85ee3b9608cd7527e13b6b2d995e",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "861040f1fbd29388f25f94f24810f44a3fb4827e60cbbbda48044ce15d0897fb",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate design
```text
{
  "change": "rustc-dev-guide-planning-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8d7be198528de12374d520314498588e7a2282aba9848066955f5590e7935f64",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "7bbf7ec031c1f3e8919326bb5ab112777d1832e7ca53991e298587eff2d0612a",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```
## post-merge cairn gate tasks
```text
{
  "change": "rustc-dev-guide-planning-boundaries",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "2fc855c8365efdd16366e897345ea5189e778c63abce3493e0fe45ae28499b28",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "cc3db8e77bb3fbfbfdbe4beae31a2b881b7b6f050584c72557115a3c903a16b3",
  "receipt_hash": "7a13101d272c6e8114494d3dad3d151c02217add6a9dba370e68133704dfc00a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive validation
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
## post-archive status
```text
## main...origin/main [ahead 3]
 M README.md
 M cairn/specs/rust-package-planning/spec.md
 M src/main.rs
?? cairn/archive/2026-07-09-rustc-dev-guide-planning-boundaries/
?? cairn/changes/adopt-cap-std-release-boundaries/
?? cairn/changes/nix-evidence-core/
?? docs/rustc-dev-guide-planning.md
?? src/rustc_dev_guide.rs
```
