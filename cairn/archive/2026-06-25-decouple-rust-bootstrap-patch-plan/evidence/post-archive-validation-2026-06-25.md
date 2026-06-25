# Post-archive validation — decouple Rust bootstrap patch plan — 2026-06-25

After `CAIRN_ARCHIVE_DATE=2026-06-25 cairn archive decouple-rust-bootstrap-patch-plan --execute`, the archived delta requirements were manually synced into `cairn/specs/rust-package-planning/spec.md` because the archive move did not update the accepted spec automatically.

## git diff --check

```text
command: git diff --check
```

## Cairn validate

```json
command: nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

## Accepted requirement sync check

```text
command: grep -n -E bootstrap_patch_plan_boundary\|provider_contract_independence cairn/specs/rust-package-planning/spec.md
3340:r[rust_package_planning.source_built_toolchain_closure.bootstrap_patch_plan_boundary] Mantle MUST isolate source-built Rust compiler-bootstrap repair decisions in a deterministic patch-plan boundary before provider materialization mutates source trees or emits generated shell.
3367:r[rust_package_planning.source_built_toolchain_closure.provider_contract_independence] Mantle MUST keep downstream Rust planning, topology execution, and self-build consumers coupled to a stable source-built toolchain provider contract rather than compiler-bootstrap implementation details.
```
