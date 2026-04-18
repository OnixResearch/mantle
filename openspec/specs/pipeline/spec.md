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
- force derivation values per root only when needed for conversion, and
- continue to use direct typed derivation extraction rather than a whole-program
  JSON export string for build execution.

`evaluate_to_json()` remains available for `crunch eval` and other debug or
reporting paths, but the build path MUST use the lazy discovery + selected-root
forcing representation that `crunch-eval` exposes.

The pipeline MUST still run `crunch_glue::convert()` for each derivation and
bridge the `ConversionCache` to a `DerivationRegistry`.

#### Scenario: Package-set build discovers labels before per-root forcing

- GIVEN a `.ncl` file exporting a record of derivations
- WHEN `build()` is called
- THEN the pipeline discovers the root labels before it begins per-root
  derivation forcing and conversion
- AND it does not need a whole-program export-ready result up front just to
  learn those labels

#### Scenario: Full-root build still works through per-root forcing

- GIVEN a `.ncl` file exporting multiple root derivations
- WHEN `build()` is called and all roots are selected for build
- THEN the pipeline may still force all roots eventually
- AND it does so through per-root forcing rather than a required whole-program
  export step before selection

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

