## ADDED Requirements

### Requirement: r[project-identity-canonical-mantle] Canonical Mantle identity

The system MUST use `Mantle` as the canonical product name and `mantle` as the canonical command/package spelling in new user-facing surfaces.

User-facing surfaces include CLI help/version text, README and operator docs, package metadata, release evidence manifests, self-build/proof summaries, and generated examples. Historical archives MAY retain Crunch/crunch references when they are clearly archival and not presented as current instructions.

#### Scenario: CLI reports canonical identity

- GIVEN the canonical binary is installed
- WHEN an operator runs `mantle --help` or `mantle --version`
- THEN the output identifies the tool as Mantle/mantle
- AND it does not present Crunch/crunch as the current product name

#### Scenario: Current docs use Mantle identity

- GIVEN a reader follows current README or docs entry points
- WHEN they read installation, build, self-build, release, or project-management instructions
- THEN the commands and product identity use Mantle/mantle
- AND legacy Crunch/crunch names appear only in explicit migration or compatibility sections

### Requirement: r[project-identity-compatibility] Crunch compatibility is explicit

The system MUST handle legacy Crunch-named entry points and files through explicit compatibility or migration behavior rather than silent mixed identity.

At minimum, the implementation MUST define behavior for:
- a legacy `crunch` command entry point, if retained,
- `crunch-project.ncl`, `crunch.lock`, and `.crunch/`,
- logical store paths rooted at `/crunch/store`,
- existing release/proof evidence created before the rename.

#### Scenario: Legacy command is transitional when retained

- GIVEN a `crunch` compatibility command is installed
- WHEN an operator invokes `crunch --help`
- THEN it delegates to the Mantle implementation or tells the operator how to migrate
- AND it identifies `mantle` as the canonical command

#### Scenario: Conflicting project files fail clearly

- GIVEN a project contains both Mantle-named and Crunch-named manifest or lock files
- WHEN `mantle check` runs
- THEN the command fails with a diagnostic that names the conflicting files
- AND it tells the operator which migration action resolves the conflict

### Requirement: r[project-identity-default-paths] Mantle defaults replace Crunch defaults

New project and store defaults MUST use Mantle-named paths unless a compatibility mode is explicitly selected.

At minimum:
- the default project manifest SHOULD be `mantle-project.ncl`,
- the default lockfile SHOULD be `mantle.lock`,
- the generated input directory SHOULD be `.mantle/`,
- the default logical store prefix SHOULD be `/mantle/store`,
- `--nix-compat` MUST continue to mean `/nix/store`, not a Crunch compatibility mode.

#### Scenario: New project initialization uses Mantle files

- GIVEN an empty directory
- WHEN `mantle init` runs
- THEN it creates Mantle-named project files and generated directories
- AND it does not create Crunch-named files unless an explicit legacy option is selected

#### Scenario: Default store prefix is Mantle

- GIVEN no explicit `--store-prefix` or compatibility option
- WHEN `mantle build hello.ncl` computes logical output paths
- THEN the logical output paths are rooted at `/mantle/store`

#### Scenario: Nix compatibility remains separate

- GIVEN an operator requests Nix compatibility
- WHEN `mantle build --nix-compat hello.ncl` runs
- THEN the logical store prefix is `/nix/store`
- AND the behavior is not described as Crunch compatibility

### Requirement: r[project-identity-proof-artifacts] New proof artifacts use Mantle identity without stronger claims

New release, attestation, parity, and self-build proof artifacts MUST identify the canonical product as Mantle while preserving the existing bounded proof semantics.

The rename MUST NOT cause docs, receipts, or verification output to claim full-source bootstrap, independent rebuild agreement, or global reproducibility unless those claims are proven by separate requirements.

#### Scenario: Release evidence records Mantle product identity

- GIVEN a new release evidence bundle is produced after the rename
- WHEN its manifest is inspected
- THEN the product identity field or equivalent manifest text names Mantle/mantle
- AND the bundle still describes only the bounded integrity and proof-context guarantees it verifies

#### Scenario: Historical proof evidence remains readable

- GIVEN a pre-rename proof or release evidence artifact names Crunch/crunch
- WHEN Mantle verification tooling reads that artifact
- THEN it either verifies it as a historical Crunch-era artifact or rejects it with a clear unsupported-version diagnostic
- AND it does not silently rewrite historical evidence identity

### Requirement: r[project-identity-stale-branding-check] Stale branding is checked deterministically

The repository MUST include a deterministic check that classifies remaining `Crunch`/`crunch` occurrences after the rename.

The check MUST fail for unclassified current user-facing old branding and MAY allow:
- archived OpenSpec changes,
- compatibility tests and migration docs,
- changelog/history entries,
- external URLs or upstream names that intentionally include `crunch`.

#### Scenario: Unclassified current branding fails

- GIVEN a current README, CLI help fixture, or release manifest fixture contains an unclassified `crunch` command reference
- WHEN the stale-branding check runs
- THEN it fails and reports the file and occurrence

#### Scenario: Archived history is allowed

- GIVEN an archived OpenSpec change contains historical Crunch wording
- WHEN the stale-branding check runs
- THEN that archived occurrence is ignored or classified as historical
