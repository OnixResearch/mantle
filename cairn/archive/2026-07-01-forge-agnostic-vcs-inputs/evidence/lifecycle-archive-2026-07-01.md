# Lifecycle Archive — Forge-agnostic VCS inputs

Date: 2026-07-01

## Sync and archive

- Sync dry-run: pueue task 1322, unblocked (`dry_run: true`, empty `reasons`).
- Sync execute: pueue task 1326, succeeded and updated `cairn/specs/project-workflows/spec.md`.
- Post-sync validate: pueue task 1330, valid with `changes: 1`, `specs_validated: 13`.
- Archive dry-run: pueue task 1333, unblocked (`blocked: false`, empty `reasons`).
- Archive execute: pueue task 1335, moved the change to `cairn/archive/2026-07-01-forge-agnostic-vcs-inputs/`.

## Post-archive validation

Command (pueue task 1342):

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 12,
  "valid": true
}
```

## Post-evidence-update validation

Command (same-turn rerun):

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 12,
  "valid": true
}

```
