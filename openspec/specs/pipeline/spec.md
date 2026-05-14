# Pipeline Crate Specification

## Purpose

Defines the `crunch-pipeline` crate: the integration layer that wires
eval, convert, and build into a single callable function.
## Requirements
### Requirement: Pipeline entry point

The system MUST provide a `build()` async function that takes a
`BuildConfig` and returns a `PipelineResult`.

`BuildConfig` MUST include:
- `file: PathBuf` — the .ncl source file
- `import_paths: Vec<OsString>` — Nickel import search paths
- `output_dir: PathBuf` — physical store directory for root outputs
- `state_dir: PathBuf` — persistent state (pathinfo.redb, blobs/)
- `store_dir: String` — logical store prefix used for drv/output path hashing
- `verbose: bool`
- `max_jobs: u32`
- `substituter_url: Option<String>`

`PipelineResult` MUST include:
- `outcomes: Vec<BuildOutcome>` — per-root-derivation build results
- `failed: Vec<FailedGoal>` — per-root-derivation failures
- `fod_mismatches: Vec<FodMismatch>` — hash mismatches for `--fix`

#### Scenario: Single derivation build

- GIVEN a .ncl file describing one derivation
- WHEN `build()` is called with a valid BuildConfig
- THEN the derivation is evaluated, converted, built, and the result
  contains one BuildOutcome with the output path

#### Scenario: Package set build

- GIVEN a .ncl file exporting a record of derivations
- WHEN `build()` is called
- THEN all derivations are built and results contain one BuildOutcome
  per root derivation

#### Scenario: Build failure with partial results

- GIVEN a .ncl file with two derivations, one of which fails
- WHEN `build()` is called
- THEN `PipelineResult.outcomes` contains the successful build and
  `PipelineResult.failed` contains the failed one

### Requirement: Pipeline owns store construction

The pipeline MUST construct `StoreHandle`, `BubblewrapBuildService`,
and `Builder` internally from `BuildConfig` paths. Callers MUST NOT
need to construct or pass service objects.

#### Scenario: Caller provides paths only

- GIVEN a `BuildConfig` with `output_dir` and `state_dir` set
- WHEN `build()` is called
- THEN the pipeline opens blob, directory, pathinfo services and
  constructs the builder without caller involvement

### Requirement: Pipeline owns eval and convert

The pipeline MUST continue to evaluate Nickel through `crunch-eval`, discover
root labels through the lazy session boundary, force derivation values per root
only when needed for conversion, and use direct typed derivation extraction
rather than a whole-program JSON export string for build execution.

The pipeline MUST treat eval execution strategy as host policy layered on top of
those `crunch-eval` semantics. It MAY choose or receive a local inline,
threaded, or subprocess backend, but it MUST NOT make one specific thread-pool
or subprocess mechanism a semantic requirement of `crunch-eval` itself.

The pipeline MUST preserve root-label association, conversion semantics, and
failure reporting across every shipped eval execution backend.

If the pipeline prefers a shipped non-inline backend on one host, it MUST still
fall back to the required inline backend when that non-inline backend is
unavailable, unsupported, or not selected.

For streaming execution, the pipeline MUST treat each root-force request as an
independent error-handling unit. It MUST NOT depend on partial success results
from a failed multi-root request.

#### Scenario: Pipeline result semantics stay stable across eval backends

- GIVEN a `.ncl` file exporting multiple root derivations
- WHEN `build()` runs with different shipped eval execution backends
- THEN the discovered root labels remain the same
- AND the converted derivations remain associated with the same labels
- AND a root-force failure reports the same failed label regardless of backend

#### Scenario: Pipeline may choose a host backend without changing eval-core contract

- GIVEN the current runtime prefers a local threaded or subprocess backend for
  performance on one host
- WHEN `build()` invokes `crunch-eval`
- THEN the pipeline may use that host policy choice
- AND the portable eval-core contract still remains valid with the required
  inline backend alone

#### Scenario: Preferred non-inline backend falls back to inline

- GIVEN the runtime prefers a shipped non-inline eval backend on one host
- AND that backend is unavailable, unsupported, or not selected for the current
  request
- WHEN `build()` invokes `crunch-eval`
- THEN the pipeline falls back to the required inline backend
- AND root-label association, conversion semantics, and labeled failure
  reporting remain unchanged

### Requirement: FOD mismatches as data

The pipeline MUST NOT abort when a fixed-output derivation produces
content with a hash that doesn't match the declared hash. It MUST
record the mismatch in `PipelineResult.fod_mismatches` and continue
building other derivations.

#### Scenario: FOD mismatch reported

- GIVEN a FOD with an incorrect hash
- WHEN the build completes
- THEN `fod_mismatches` contains a `FodMismatch` with `name`,
  `expected_sri`, and `actual_sri`

### Requirement: No CLI concerns

The pipeline crate MUST NOT depend on `clap`. It MUST NOT write to
stdout or stderr. It MUST NOT read environment variables for CLI
configuration. It MUST NOT perform `--fix` source rewriting.
The caller is responsible for resolving `state_dir` before constructing
`BuildConfig`.

#### Scenario: No direct output

- GIVEN any `build()` call
- WHEN the pipeline runs
- THEN all user-facing output is returned as data in `PipelineResult`,
  not printed

### Requirement: Crate layout

The workspace MUST contain the following crates:

| Crate | Role |
|---|---|
| `mantle` (binary) | CLI entry point, arg parsing, error formatting, log writing |
| `crunch-pipeline` | Eval->convert->build integration, store/builder construction |
| `crunch-eval` | Nickel evaluation wrapper |
| `crunch-glue` | CrunchDerivation -> nix_compat::Derivation conversion |
| `crunch-build` | Goal scheduler, build dispatch, output processing |
| `crunch-store` | StoreHandle, cache checking, castore export, queries |

#### Scenario: Workspace exposes the expected crate split

- GIVEN the workspace manifest and crate directories
- WHEN the project layout is inspected
- THEN the workspace contains `mantle`, `crunch-pipeline`, `crunch-eval`,
  `crunch-glue`, `crunch-build`, and `crunch-store`
- AND each crate owns the role listed above

