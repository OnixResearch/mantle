# Archive validation evidence

Date: 2026-06-18
Archived change: first-stage-musl-proc-macro-runtime

The Cairn archive command moved the active change to this archive. The ADDED requirement did not auto-sync into the accepted spec, so it was manually merged into `cairn/specs/rust-package-planning/spec.md` before this validation.

## accepted spec contains requirement

```text
$ grep -n first_stage_musl_proc_macro_runtime cairn/specs/rust-package-planning/spec.md
3244:r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime] Mantle MUST make source-root musl runtime libraries visible to first-stage musl-host proc macros without exposing generic host aliases globally.

```

## cairn validate after archive and manual spec sync

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

## git diff --check after archive

```text
$ git diff --check

```

