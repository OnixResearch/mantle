# Archive validation evidence (2026-06-18)

Task-ID: archive
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_crt_normalization]

## Archive command

```sh
CAIRN_ARCHIVE_DATE=2026-06-18 nix run path:/home/brittonr/git/cairn#cairn -- archive first-stage-musl-static-crt-normalization --root . --execute
```

Cairn moved the active change to:

```text
cairn/archive/2026-06-18-first-stage-musl-static-crt-normalization/
```

## Manual spec sync note

As with prior Cairn archive gotchas in this repo, the archive move did not copy the ADDED requirement into the accepted spec. I manually merged the requirement into:

```text
cairn/specs/rust-package-planning/spec.md
```

Requirement now present:

```text
### Requirement: First-stage musl static CRT normalization

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_crt_normalization] Mantle MUST pair first-stage source-root musl static link mode normalization with a matching non-PIE musl startup object.
```

## Post-archive validation

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Result (pueue task `58`, excerpt):

```json
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
