# Archive validation

Task-ID: V3
Covers: rust_package_planning.source_built_toolchain_closure.provider_status

## Pre-archive validation

Pueue task 24 ran:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-toolchain-closure-proof --root .
nix run path:/home/brittonr/git/cairn#cairn -- sync source-built-toolchain-closure-proof --root .
```

Visible sync dry-run excerpt:

```json
{
  "change": "source-built-toolchain-closure-proof",
  "delta_specs": [
    "./cairn/changes/source-built-toolchain-closure-proof/specs/rust-package-planning/spec.md"
  ],
  "dry_run": true,
  "mutated": false,
  "reasons": []
}
```

Pueue task 25 ran:

```text
nix run path:/home/brittonr/git/cairn#cairn -- sync source-built-toolchain-closure-proof --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Visible validation excerpt:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

## Accepted spec check

`cairn/specs/rust-package-planning/spec.md` now contains `r[rust_package_planning.source_built_toolchain_closure.provider_status]` with scenarios for provider-backed status, absent-provider non-claim retention, explicit manifest authority, and broader proof bounds.

## Post-archive validation

Pueue task 26 ran tasks gate, archive execute, and validation. Visible post-archive validation excerpt:

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
