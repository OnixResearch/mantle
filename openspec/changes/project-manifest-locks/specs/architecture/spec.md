## MODIFIED Requirements

### Requirement: Crate layout

The workspace MUST contain the following crates:

| Crate | Role |
|---|---|
| `crunch` (binary) | CLI parsing, error formatting, log writing, bootstrap, self-build dispatch |
| `crunch-project` | Project manifest, lockfile, refresh, stale detection, upgrade, generated inputs |
| `crunch-pipeline` | Eval->convert->build integration, store/builder construction |
| `crunch-eval` | Nickel evaluation wrapper |
| `crunch-glue` | CrunchDerivation -> nix_compat::Derivation conversion |
| `crunch-build` | Goal scheduler, build dispatch, output processing |
| `crunch-store` | StoreHandle, cache checking, castore export, queries |
| vendored crates | Data layer (nix-compat, snix-build, snix-castore, snix-store) |

#### Scenario: Project command uses dedicated crate

- GIVEN a project-management CLI command such as `crunch refresh`
- WHEN the binary handles the command
- THEN it delegates to `crunch-project`
- AND the binary does not own manifest, lock, or refresh logic itself

## ADDED Requirements

### Requirement: Project-management layer is separate from the build engine

The project-management layer MUST stay separate from the eval/build/store
engine.

`crunch-project` MAY load manifests, compute stale state, rewrite lockfiles,
and generate `.crunch/inputs.ncl`, but it MUST NOT become a second fetch or
build engine.

Network fetch execution, fixed-output verification, patch application, sandbox
execution, and store persistence MUST continue to live in the existing
fetcher/build/store layers.

#### Scenario: Project layer does not duplicate fetch execution

- GIVEN an input described in `crunch-project.ncl`
- WHEN the project layer resolves and materializes that input
- THEN it records the metadata needed by the build path
- AND actual fetch/build execution still flows through crunch's existing
  fetcher and pipeline crates
