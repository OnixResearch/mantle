# Archive validation evidence

Date: 2026-06-18

## manual spec sync

The Cairn archive moved the change package but did not copy the ADDED requirement into `cairn/specs/rust-package-planning/spec.md`. The requirement was manually inserted beside the other first-stage musl requirements before this validation.

## accepted-spec grep

```text
3218:### Requirement: First-stage musl rustc-driver rlib normalization
3220:r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib] Mantle MUST normalize the Rust 1.90 `rustc_driver` crate type for the source-root musl compiler-host first-stage build.
```

## git diff --check

```text
```

## cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}
```
