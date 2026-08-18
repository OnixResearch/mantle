# Post-archive validation

Date: 2026-08-09

The legacy archive command created `cairn/archive/1970-01-01-preserve-nix-fetch-mirror-order/`. The directory was renamed to `cairn/archive/2026-08-09-preserve-nix-fetch-mirror-order/`.

The legacy command moved the package without synchronizing its accepted requirement. The requirement was appended once to `cairn/specs/foreign-derivation-import/spec.md`.

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

The complete output is in `post-archive-validation.json`.

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- tracey coverage --root .
```

Exact output:

```text
traceability coverage ok: 155/155 referenced (profile mantle-default)
```

The complete output is in `post-archive-tracey.log`.

This archive does not claim arbitrary Nix fetcher parity, mirror trust, Nix evaluation, Nixpkgs parity, source correctness, build correctness, or release eligibility.
