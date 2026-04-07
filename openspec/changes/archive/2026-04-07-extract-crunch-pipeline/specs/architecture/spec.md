# Architecture Specification — Delta

## MODIFIED Requirements

### Requirement: Crate layout

The workspace MUST contain the following crates:

| Crate | Role |
|---|---|
| `crunch` (binary) | CLI parsing, error formatting, log writing, bootstrap, self-build dispatch |
| `crunch-pipeline` | Eval->convert->build integration, store/builder construction |
| `crunch-eval` | Nickel evaluation wrapper |
| `crunch-glue` | CrunchDerivation -> nix_compat::Derivation conversion |
| `crunch-build` | Goal scheduler, build dispatch, output processing |
| `crunch-store` | StoreHandle, cache checking, castore export, queries |
| vendored crates | Data layer (nix-compat, snix-build, snix-castore, snix-store) |

#### Scenario: Library embedding

- GIVEN a Rust program that depends on `crunch-pipeline`
- WHEN it calls `crunch_pipeline::build(&config).await`
- THEN a build executes and returns structured results
- AND no CLI parsing or terminal output occurs

#### Scenario: Binary delegates to pipeline

- GIVEN `crunch build hello.ncl`
- WHEN the binary processes the command
- THEN it constructs a `BuildConfig` and ultimately calls `crunch_pipeline::build(&config)`
- AND formats the returned `PipelineResult` for the terminal

### Requirement: Pipeline stages

Pipeline stages are unchanged, but the wiring between stages
MUST live in `crunch-pipeline` for the standard build path:

1. Nickel source -> evaluated JSON (crunch-eval)
2. JSON -> Derivation structs (crunch-glue)
3. Derivation stream -> Worker dispatch (crunch-pipeline)
4. Worker -> BuildService (crunch-build)
5. BuildResult -> PathInfo persistence (crunch-store)

The binary crate MUST delegate the standard build path (`crunch build`
and the build portion of `crunch self-build`) to `crunch-pipeline`.
It MAY still call lower layers directly for helper commands that are
outside the standard build path, including `crunch eval`, bootstrap
fetch/staging helpers, and `crunch store list/info/verify`.

#### Scenario: Helper commands keep direct lower-layer calls

- GIVEN a helper command that is not the standard build path
- WHEN the binary handles `crunch eval`, bootstrap fetch/staging work,
  or `crunch store` queries
- THEN it MAY call the lower crates directly
- AND that does not violate the pipeline extraction requirement
