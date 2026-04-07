## ADDED Requirements

### Requirement: Project-management commands

The CLI MUST provide project-management commands in addition to the existing
engine-oriented commands.

Required commands:
- `crunch init`
- `crunch check`
- `crunch show`
- `crunch refresh`
- `crunch list-stale`
- `crunch upgrade`

These commands operate on the project manifest, lockfile, and generated input
files. They MUST delegate to the project-management layer rather than embed
that logic directly in `src/main.rs`.

#### Scenario: Init scaffolds project files

- GIVEN a directory without crunch project files
- WHEN `crunch init` runs
- THEN it creates `crunch-project.ncl`
- AND it creates `crunch.lock`
- AND it creates or documents the generated `.crunch/` directory layout

#### Scenario: Check validates project state

- GIVEN a project with `crunch-project.ncl`, `crunch.lock`, and `.crunch/inputs.ncl`
- WHEN `crunch check` runs
- THEN it validates the manifest and lockfile
- AND it reports drift or schema errors with a non-zero exit code

#### Scenario: Refresh updates selected inputs

- GIVEN a project with multiple named inputs
- WHEN `crunch refresh foo bar` runs
- THEN only those named inputs are refreshed
- AND `crunch.lock` and `.crunch/inputs.ncl` are rewritten if their resolved
  state changes

#### Scenario: Show renders resolved input state

- GIVEN a valid project manifest and lockfile
- WHEN `crunch show` runs
- THEN it prints a human-readable view of the resolved inputs, including
  frozen state, mirrors, patches, and locked revisions or hashes

#### Scenario: Upgrade migrates project files

- GIVEN a project using an older supported schema version
- WHEN `crunch upgrade` runs
- THEN the project manifest and lockfile are migrated to the current version
