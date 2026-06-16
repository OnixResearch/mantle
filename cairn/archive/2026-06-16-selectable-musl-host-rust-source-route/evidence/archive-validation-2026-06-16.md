# Archive validation (2026-06-16)

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.selectable_rust_source_route]

## Pre-archive validation

Active change validation passed in pueue task 22 with `cairn validate`, `cairn gate proposal`, `cairn gate design`, and `cairn gate tasks`; the final gate verdict was `PASS` with `stage: "tasks"`.

## Archive command and post-archive validation

Pending until archive execution completes.

## Archive notes

- `CAIRN_ARCHIVE_DATE=2026-06-16 cairn archive --execute` moved the change to `cairn/archive/2026-06-16-selectable-musl-host-rust-source-route`.
- The archive command moved the change but did not sync the ADDED requirement into the canonical spec, so `selectable_rust_source_route` was manually copied into `cairn/specs/rust-package-planning/spec.md` before validation.

## Post-archive command

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
status=0
```

### stdout

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

### stderr

```text
```
