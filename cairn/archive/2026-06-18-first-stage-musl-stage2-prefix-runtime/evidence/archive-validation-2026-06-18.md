# Archive validation evidence

Date: 2026-06-18

The first archive command created `cairn/archive/1970-01-01-first-stage-musl-stage2-prefix-runtime`; it was manually renamed to `cairn/archive/2026-06-18-first-stage-musl-stage2-prefix-runtime`. The ADDED requirement did not sync automatically, so it was manually copied into `cairn/specs/rust-package-planning/spec.md` before validation.

## archived path and synced requirement

```text
$ test -d cairn/archive/2026-06-18-first-stage-musl-stage2-prefix-runtime

$ grep -n first_stage_musl_stage2_prefix_runtime\|First-stage musl stage2 prefix runtime cairn/specs/rust-package-planning/spec.md
3289:### Requirement: First-stage musl stage2 prefix runtime visibility
3291:r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_prefix_runtime] Mantle MUST include the stage2 rustc prefix runtime directory in source-root musl first-stage Cargo probes.

```

## post-archive cairn validate

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
