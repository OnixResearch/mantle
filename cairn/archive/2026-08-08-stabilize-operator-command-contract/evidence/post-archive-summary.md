# Post-archive validation

Date: 2026-08-08

The legacy archive command created a `1970-01-01` directory. The directory was
renamed to `2026-08-08-stabilize-operator-command-contract` as required by the
repository workflow.

The archive command moved the change but did not synchronize its ADDED
requirements. The five accepted requirements were appended to
`cairn/specs/operator-diagnostics/spec.md` without changing existing
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
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
```

The complete JSON output is in `post-archive-validation.json`.

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- tracey coverage --root .
```

Exact output:

```text
traceability coverage ok: 155/155 referenced (profile mantle-default)
```

No source-built fixed-point or full-source completion claim is part of this
archive.
