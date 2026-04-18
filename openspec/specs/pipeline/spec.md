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

The pipeline MUST evaluate Nickel in-process through `crunch-eval`, but it
MUST NOT require a whole-program export-ready deep evaluation merely to obtain
root labels.

The build path MUST:
- discover root labels through the lazy `crunch-eval` boundary first,
- force derivation values per root only when needed for conversion,
- allow bounded overlap between multi-root forcing/conversion and downstream
  worker scheduling when more than one root is requested, and
- continue to use direct typed derivation extraction rather than a whole-program
  JSON export string for build execution.

`evaluate_to_json()` remains available for `crunch eval` and other debug or
reporting paths, but the build path MUST use the lazy discovery + selected-root
forcing representation that `crunch-eval` exposes.

The pipeline MUST still run `crunch_glue::convert()` for each derivation and
bridge the `ConversionCache` to a `DerivationRegistry`.

The pipeline MUST serialize `crunch_glue::convert()` calls against the shared
`ConversionCache` and `DerivationRegistry`, and it MUST pass the configured
store prefix to all conversion call sites.

The first iteration of pipeline-side eval parallelism MUST be bounded by an
internal cap no greater than the configured build parallelism. When the build
configuration does not set `max_jobs`, the pipeline MUST derive its build and
eval caps from the existing default build-parallelism resolution and still keep
eval parallelism at least `1`.

For streaming execution, the pipeline MUST treat each root-force request as an
independent error-handling unit. It MUST NOT depend on partial success results
from a failed multi-root request.

#### Scenario: Multi-root build streams converted roots incrementally

- GIVEN a `.ncl` file exporting multiple root derivations
- WHEN `build()` is called
- THEN the pipeline discovers root labels before per-root forcing begins
- AND it sends converted roots to the worker incrementally as they are ready
- AND it does not wait for a full serial evaluated-root vector before worker
  scheduling can start
- AND it preserves label -> derivation association even if converted roots are
  streamed in completion order rather than request order

#### Scenario: Eval parallelism stays bounded by build parallelism

- GIVEN `build()` runs with a configured `max_jobs`
- WHEN the pipeline chooses its eval parallelism
- THEN the eval parallelism does not exceed that configured build parallelism
- AND the pipeline does not create unbounded root-force tasks

#### Scenario: Root forcing failure stops later dispatch without losing prior labels

- GIVEN the pipeline has already streamed one converted root to the worker
- AND a later independent root-force request fails during bounded evaluation
- WHEN `build()` surfaces that evaluation failure
- THEN the pipeline stops dispatching additional not-yet-converted roots
- AND it preserves label association for any root already sent downstream
- AND the final build result reports the labeled evaluation failure

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
| `crunch` (binary) | CLI entry point, arg parsing, error formatting, log writing |
| `crunch-pipeline` | Eval->convert->build integration, store/builder construction |
| `crunch-eval` | Nickel evaluation wrapper |
| `crunch-glue` | CrunchDerivation -> nix_compat::Derivation conversion |
| `crunch-build` | Goal scheduler, build dispatch, output processing |
| `crunch-store` | StoreHandle, cache checking, castore export, queries |

#### Scenario: Workspace exposes the expected crate split

- GIVEN the workspace manifest and crate directories
- WHEN the project layout is inspected
- THEN the workspace contains `crunch`, `crunch-pipeline`, `crunch-eval`,
  `crunch-glue`, `crunch-build`, and `crunch-store`
- AND each crate owns the role listed above

