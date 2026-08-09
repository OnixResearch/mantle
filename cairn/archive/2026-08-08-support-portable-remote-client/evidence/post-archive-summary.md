# Post-archive validation

Date: 2026-08-08

The legacy archive command created
`cairn/archive/1970-01-01-support-portable-remote-client/`. The directory was
renamed to `cairn/archive/2026-08-08-support-portable-remote-client/`.

The legacy command moved the package without synchronizing its eight ADDED
requirements. Those requirements were appended to
`cairn/specs/realization-routing/spec.md` without changing existing
requirements.

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- validate --root .
```

Exact verdict fields:

```json
{
  "findings": [],
  "issues": [],
  "valid": true
}
```

The complete command output is in `post-archive-validation.json`.

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- tracey coverage --root .
```

Exact output:

```text
traceability coverage ok: 155/155 referenced (profile mantle-default)
```

The complete command output is in `post-archive-tracey.log`.

This archive does not claim native Darwin test evidence, fixed-point success,
full-source completion, bootstrap closure, or release readiness.
