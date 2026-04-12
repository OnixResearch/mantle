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

### Requirement: Critical-path orchestrators stay narrow

Top-level shell/orchestrator functions in critical paths MUST remain thin
coordinators over dedicated helpers instead of accumulating unrelated phases in
one body.

This requirement applies at minimum to CLI dispatch and self-build
orchestration. The parent function MAY keep phase ordering and top-level error
mapping, but phase-specific work MUST live in named helpers with narrower
responsibility boundaries.

#### Scenario: CLI dispatch delegates command-specific work

- GIVEN the binary handles a user command
- WHEN top-level dispatch runs
- THEN the dispatcher selects the command path and delegates to focused helpers
- AND command-specific evaluation, project resolution, build execution, or
  self-build staging does not remain in one monolithic dispatcher body

#### Scenario: Self-build orchestration delegates stages

- GIVEN `crunch self-build` runs
- WHEN the command advances through source staging, bootstrap-tool builds,
  crunch build, and verification
- THEN each stage is implemented by a dedicated helper or helper cluster
- AND the parent function remains responsible only for sequencing, shared
  config, and final reporting

### Requirement: Critical-path logic separates planning from effects

Critical-path logic that decides build identity or persistence inputs MUST split
pure planning from effectful execution.

At minimum this applies to content-addressed output finalization and related
attestation graph construction. Pure helpers MUST compute rewrite plans,
resolved names, validation outcomes, or graph inputs from plain values. Shell
helpers MUST perform blob writes, directory writes, pathinfo persistence,
registry mutation, or logging.

#### Scenario: CA finalization computes plan before store mutation

- GIVEN a content-addressed output is being finalized
- WHEN crunch computes marker rewrites, final output names, or other derived
  planning data
- THEN that planning step runs before store mutation
- AND the planning helper can be tested without blob, directory, or pathinfo
  services
- AND persistence and logging consume the planning result afterward

### Requirement: Critical tree traversals are iterative and bounded

Tree-shaped traversals in critical build/store paths MUST use explicit bounded
iteration rather than recursive async helper calls.

At minimum this applies to castore rewrite and export traversal. The traversal
MUST maintain an explicit worklist or equivalent bounded state and MUST fail
with a clear limit error when the configured bound is exceeded.

#### Scenario: Deep castore tree hits explicit traversal limit

- GIVEN a castore tree deeper than the supported traversal bound
- WHEN crunch rewrites or exports that tree
- THEN the traversal fails with a clear limit error
- AND the failure comes from the explicit traversal bound rather than stack
  growth from recursive helper calls

### Requirement: Critical-path helpers expose local contracts

Non-trivial helpers in the critical paths covered by this change MUST make
local safety assumptions executable with assertions or compile-time checks.

At minimum the covered code MUST assert key length relationships, required
non-empty collections, fixed worklist bounds, and other invariants that would
turn silent corruption into a loud failure.

#### Scenario: Invalid CA marker plan fails loudly

- GIVEN a critical-path helper receives inputs that violate a required length
  or count invariant
- WHEN the helper computes its planning or traversal state
- THEN it fails loudly through an assertion or explicit error
- AND the code does not continue with silently inconsistent state
