# Tracey bridge oracle checkpoint

Question: Does `tools/tracey_refs.rs` honestly close the Tracey linkage gap for `rust_package_planning.source_built_toolchain_closure` after archive sync, without claiming global Tracey coverage is green?

Inspected evidence:

- Canonical requirement: `cairn/specs/rust-package-planning/spec.md:2809`
- Bridge refs: `tools/tracey_refs.rs:8` and `tools/tracey_refs.rs:12`
- Archived fixed-point proof evidence: `cairn/archive/2026-05-31-source-built-toolchain-closure/evidence/source-built-closure-fixed-point-validation.md`
- Current command output below.

Command output:

```text
## status
## main...origin/main [ahead 15]
## validate
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
## tracey-json
error: tracey coverage failed
{
  "valid": false,
  "requirements": 202,
  "referenced": 1,
  "target_missing": null,
  "missing_count": 201,
  "dangling": []
}
## bridge-refs
cairn/archive/2026-05-31-source-built-toolchain-closure/evidence/source-built-closure-fixed-point-validation.md:4:Covers: rust_package_planning.source_built_toolchain_closure
tools/tracey_refs.rs:8:// r[impl rust_package_planning.source_built_toolchain_closure]
tools/tracey_refs.rs:12:// r[verify rust_package_planning.source_built_toolchain_closure]
cairn/specs/rust-package-planning/spec.md:2809:r[rust_package_planning.source_built_toolchain_closure] Mantle MUST provide a separate audit-grade proof before claiming that a Cargo-free self-build or fixed-point run used a source-built compiler/toolchain closure.
```

Decision: Yes, for the narrow post-archive linkage claim only. The bridge refs make `rust_package_planning.source_built_toolchain_closure` no longer appear in Cairn Tracey coverage's missing list (`target_missing: null`). This does **not** claim global Tracey coverage is green: the repo-wide rail remains red with `missing_count: 201` and `valid: false`.

Owner: Mantle maintainers.

Next action: Treat global Tracey coverage as separate debt. Do not use this bridge checkpoint as proof for the other 201 missing requirements.
