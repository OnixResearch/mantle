## Why

`crunch shell` exists and works — it builds a shell derivation, reads a sidecar JSON, computes an activation plan, and execs into the result. But it requires the user to know which derivation to point at. There is no `crunch develop` command that reads a project-local `crunch-project.ncl`, finds the devShell definition, builds it, and drops into it.

The scaffolding is already there: `src/project_build.rs` has devShell extraction logic, `crunch shell` handles activation, and `crunch-project` manages the manifest. The gap is wiring them into a single `crunch develop` command.

## What Changes

- **`crunch develop` command**: Reads `crunch-project.ncl` in the current directory (or `--project <path>`), extracts the default devShell (or a named one via `crunch develop <name>`), builds it, and activates it through the existing `crunch shell` path.
- **Project manifest devShell schema**: `crunch-project.ncl` gains a `devShells` field (or formalizes the existing one) where projects declare named shell environments with their dependencies.
- **Automatic rebuild on change**: Optional `--watch` flag that re-evaluates and re-activates when `crunch-project.ncl` changes (stretch goal, not required for v1).

## Capabilities

### New Capabilities
- `develop`: Single command to enter a project's dev environment
- `develop-named`: Support for multiple named devShells per project

### Modified Capabilities
- `shell`: No changes — `crunch develop` delegates to the existing shell activation path
- `project`: Manifest schema extended with devShell declarations

## Impact

- **Files**: `src/main.rs` (new subcommand), `src/project_build.rs` (devShell resolution), `lib/project.ncl` (schema)
- **APIs**: New CLI subcommand only — no library API changes
- **Dependencies**: None
- **Testing**: Integration test that `crunch develop` in a fixture project produces a working shell with expected tools on PATH
