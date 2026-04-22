# Architecture Specification

## Purpose

Defines crunch workspace structure, crate responsibilities, and critical-path
boundary rules so project-management, build, and functional-core ownership stay
explicit as the workspace evolves.
## Requirements
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

### Requirement: No-std core crates are a first-class workspace tier

The workspace MUST distinguish dedicated no-std core crates from std
shell/adaptor crates across the adopted no-std waves.

ID: architecture.nostd.core.workspace.tier

The adopted no-std waves MUST contain these crates and roles:

| Crate | Role |
|---|---|
| `crunch-attestation-core` | no-std ownership of logic currently implemented in `crates/crunch-attestation/src/{canonical.rs,digest.rs,error.rs,policy.rs,release.rs,schema.rs,version.rs}`, including `evaluate_policy(...)` and `binary_digests_match(...)` |
| `crunch-project-core` | no-std ownership of logic currently implemented in `crates/crunch-project/src/{manifest.rs,lock.rs,merge.rs,drift.rs,upgrade.rs,version.rs,generate.rs,refresh.rs}` after the refresh path is split so core owns `ResolvedInput`, `HashResolutionMode`, `RefreshFailure`, `StaleReport`, `RefreshOutcome`, `ApplyResult`, `refresh_inputs(...)`, `apply_outcomes(...)`, and `list_stale(...)` |
| `crunch-shell-core` | no-std ownership of shell sidecar JSON validation plus activation env/path/hook planning over owned UTF-8 data |
| `crunch-release-core` | no-std ownership of release-evidence manifest canonicalization, bundle-member/path validation, proof-linkage validation, and full self-hosting proof identity parsing over owned bytes/strings |
| `crunch-attestation` | std adaptor/re-export layer that retains `crates/crunch-attestation/src/discovery.rs`, file discovery, and release-bundle loading before calling core logic |
| `crunch-project` | std adaptor/re-export layer that retains `RefreshResolver` implementations, resolver I/O, tempdirs, file reads/writes, and shell-facing project APIs around the moved project-core logic |
| `crunch-shell` | std adaptor layer that retains `PathBuf`, `OsString`, `split_paths(...)`, non-UTF-8 rejection, and `ExecTarget` reconstruction around `crunch-shell-core` |
| `crunch` | std shell for release evidence that retains `src/release_evidence.rs`, `src/release_cmd.rs`, file copying, directory hashing, proof-bundle loading, manifest I/O, and CLI formatting around `crunch-release-core` |

#### Scenario: Workspace shows explicit no-std core tier
ID: architecture.nostd.core.workspace.tier.visible

- GIVEN the workspace manifest and crate directories
- WHEN the project layout is inspected after this change lands
- THEN the workspace contains `crunch-attestation-core`,
  `crunch-project-core`, `crunch-shell-core`, and `crunch-release-core` as
  dedicated no-std crates
- AND the existing std-facing `crunch-attestation`, `crunch-project`,
  `crunch-shell`, and root `crunch` release-evidence path remain present as
  shell/adaptor layers

### Requirement: Adopted no-std core crates use crate boundaries for shell separation

The adopted first- and second-wave functional core MUST rely on crate
boundaries, not convention alone, to keep moved business logic separate from
imperative shell code.

ID: architecture.nostd.core.crate.boundary

Code that performs filesystem access, subprocess execution, environment reads,
clock access, network I/O, or log writing MUST live outside the adopted
no-std core crates. Core crates MUST expose deterministic transforms over plain
data and typed errors only.

Compliance with this requirement MUST be proven by
`functional.core.nostd.boundary.continuously.verified`.

For the adopted waves, deterministic means no ambient randomness and no hidden
mutable global state inside the no-std core crates.

#### Scenario: Effectful dependency stays out of first-wave core crates
ID: architecture.nostd.core.crate.boundary.effectful.dependency.outside

- GIVEN a future change that needs git subprocesses, HTTP downloads, PATH
  splitting, shell-target reconstruction, proof-bundle loading, or release
  bundle hashing for one of the adopted no-std domains
- WHEN that code is added to crunch
- THEN it lands in a std shell/adaptor crate or std root module
- AND the no-std core crate interface accepts only normalized request data and
  returns typed results

